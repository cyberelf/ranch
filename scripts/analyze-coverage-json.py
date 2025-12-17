#!/usr/bin/env python3
"""
Analyze Tarpaulin coverage report (JSON format) to find uncovered functions

This script parses the target/tarpaulin/coverage.json file and extracts:
1. Files with coverage statistics
2. Uncovered lines (lines with 0 hits)
3. Summary of coverage by file
"""

import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict

def load_coverage_json(json_file: Path) -> dict:
    """Load coverage data from JSON file"""
    with open(json_file, 'r') as f:
        return json.load(f)

def analyze_file_coverage(file_path: str, traces: List[dict]) -> Dict[str, any]:
    """Analyze coverage for a single file"""
    total_lines = len(traces)
    covered_lines = sum(1 for trace in traces if trace.get('stats', {}).get('Line', 0) > 0)
    uncovered_lines = total_lines - covered_lines
    
    coverage_percent = (covered_lines / total_lines * 100) if total_lines > 0 else 0
    
    # Get uncovered line numbers
    uncovered_line_nums = [trace['line'] for trace in traces if trace.get('stats', {}).get('Line', 0) == 0]
    
    return {
        'path': file_path,
        'total_lines': total_lines,
        'covered_lines': covered_lines,
        'uncovered_lines': uncovered_lines,
        'coverage_percent': coverage_percent,
        'uncovered_line_numbers': uncovered_line_nums
    }

def extract_function_at_line(file_path: Path, line_num: int) -> str:
    """Extract function name near the given line number"""
    if not file_path.exists():
        return "unknown"
    
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
        
        # Look backwards from the line to find function definition
        for i in range(max(0, line_num - 1), max(0, line_num - 30), -1):
            if i < len(lines):
                line = lines[i]
                # Match Rust function definitions
                fn_match = re.search(r'\b(pub\s+)?(\basync\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)', line)
                if fn_match:
                    return fn_match.group(3)
                
                # Match impl blocks
                impl_match = re.search(r'impl\s+(?:.*?\s+for\s+)?([a-zA-Z_][a-zA-Z0-9_<>]*)', line)
                if impl_match:
                    return f"impl {impl_match.group(1)}"
    
    except Exception as e:
        pass
    
    return "unknown"

def group_uncovered_by_function(file_path: str, uncovered_lines: List[int]) -> Dict[str, List[int]]:
    """Group uncovered lines by function"""
    func_lines = defaultdict(list)
    file_abs_path = Path(file_path)
    
    for line_num in uncovered_lines[:50]:  # Limit to first 50 uncovered lines
        func_name = extract_function_at_line(file_abs_path, line_num)
        func_lines[func_name].append(line_num)
    
    return dict(func_lines)

def print_summary(coverage_data: dict):
    """Print coverage summary"""
    print("=" * 80)
    print("TARPAULIN COVERAGE ANALYSIS")
    print("=" * 80)
    print()
    
    # Get files from coverage data
    traces = coverage_data.get('traces', {})
    
    # Analyze each file
    all_analyses = []
    for file_path, file_traces in traces.items():
        if file_path.endswith('.rs') and file_traces:  # Only analyze Rust files with traces
            analysis = analyze_file_coverage(file_path, file_traces)
            all_analyses.append(analysis)
    
    # Overall statistics
    if all_analyses:
        total_all_lines = sum(a['total_lines'] for a in all_analyses)
        covered_all_lines = sum(a['covered_lines'] for a in all_analyses)
        overall_coverage = (covered_all_lines / total_all_lines * 100) if total_all_lines > 0 else 0
        
        print(f"Overall Coverage: {overall_coverage:.2f}%")
        print(f"Total Coverable Lines: {total_all_lines}")
        print(f"Covered Lines: {covered_all_lines}")
        print(f"Uncovered Lines: {total_all_lines - covered_all_lines}")
        print()
        
        # Per-file breakdown
        print("-" * 80)
        print("PER-FILE COVERAGE (sorted by coverage percentage)")
        print("-" * 80)
        print()
        
        # Sort by coverage percentage (lowest first)
        sorted_analyses = sorted(all_analyses, key=lambda x: x['coverage_percent'])
        
        for analysis in sorted_analyses:
            if analysis['total_lines'] > 0:  # Only show files with coverable lines
                # Shorten path for display
                short_path = analysis['path']
                if len(short_path) > 70:
                    parts = short_path.split('/')
                    short_path = '/'.join(['...'] + parts[-3:])
                
                print(f"File: {short_path}")
                print(f"  Coverage: {analysis['coverage_percent']:.2f}% ({analysis['covered_lines']}/{analysis['total_lines']} lines)")
                
                if analysis['uncovered_lines'] > 0:
                    print(f"  Uncovered: {analysis['uncovered_lines']} lines")
                    
                    # Group by functions
                    func_groups = group_uncovered_by_function(analysis['path'], analysis['uncovered_line_numbers'])
                    if func_groups:
                        print(f"  Uncovered functions/areas ({len(func_groups)} areas):")
                        for func_name, lines in list(func_groups.items())[:5]:  # Show top 5
                            line_ranges = summarize_line_ranges(lines)
                            print(f"    - {func_name}: lines {line_ranges}")
                print()
    else:
        print("No coverage data found")

def summarize_line_ranges(lines: List[int]) -> str:
    """Summarize line numbers into ranges"""
    if not lines:
        return ""
    
    sorted_lines = sorted(lines)
    ranges = []
    start = sorted_lines[0]
    end = sorted_lines[0]
    
    for line in sorted_lines[1:]:
        if line == end + 1:
            end = line
        else:
            if start == end:
                ranges.append(str(start))
            else:
                ranges.append(f"{start}-{end}")
            start = line
            end = line
    
    # Add the last range
    if start == end:
        ranges.append(str(start))
    else:
        ranges.append(f"{start}-{end}")
    
    return ", ".join(ranges[:3]) + (" ..." if len(ranges) > 3 else "")

def main():
    # Find the coverage.json file
    coverage_file = Path(__file__).parent.parent / "target" / "tarpaulin" / "coverage.json"
    
    if not coverage_file.exists():
        print(f"Error: Could not find {coverage_file}")
        print("Run `cargo tarpaulin` first to generate coverage data")
        sys.exit(1)
    
    print(f"Analyzing: {coverage_file}")
    print()
    
    # Load coverage data
    try:
        data = load_coverage_json(coverage_file)
    except Exception as e:
        print(f"Error loading coverage data: {e}")
        sys.exit(1)
    
    # Print summary
    print_summary(data)

if __name__ == "__main__":
    main()
