#!/usr/bin/env python3
"""
Analyze Tarpaulin coverage report to find uncovered functions

This script parses the tarpaulin-report.html file and extracts:
1. Files with coverage statistics
2. Uncovered lines (lines with 0 hits)
3. Summary of coverage by file
"""

import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple

def extract_var_data(html_file: Path) -> dict:
    """Extract the JavaScript 'var data' object from the HTML file"""
    with open(html_file, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Find the line with var data = 
    # The data is on a single line, but very long
    data_start = content.find('var data = ')
    if data_start == -1:
        print("Error: Could not find 'var data' in HTML file")
        sys.exit(1)
    
    # Find the end of the var data statement (semicolon)
    json_start = data_start + len('var data = ')
    json_end = content.find(';', json_start)
    
    if json_end == -1:
        print("Error: Could not find end of 'var data' statement")
        sys.exit(1)
    
    json_str = content[json_start:json_end]
    
    try:
        return json.loads(json_str)
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}")
        print(f"JSON string length: {len(json_str)}")
        print(f"First 500 chars: {json_str[:500]}")
        sys.exit(1)

def analyze_file_coverage(file_data: dict) -> Dict[str, any]:
    """Analyze coverage for a single file"""
    path = '/'.join(file_data['path'])
    content = file_data['content']
    traces = file_data.get('traces', [])
    
    # Calculate coverage statistics
    total_lines = len(traces)
    covered_lines = sum(1 for trace in traces if trace['stats']['Line'] > 0)
    uncovered_lines = total_lines - covered_lines
    
    coverage_percent = (covered_lines / total_lines * 100) if total_lines > 0 else 0
    
    # Get uncovered line numbers
    uncovered_line_nums = [trace['line'] for trace in traces if trace['stats']['Line'] == 0]
    
    return {
        'path': path,
        'total_lines': total_lines,
        'covered_lines': covered_lines,
        'uncovered_lines': uncovered_lines,
        'coverage_percent': coverage_percent,
        'uncovered_line_numbers': uncovered_line_nums,
        'content': content
    }

def extract_function_at_line(content: str, line_num: int) -> str:
    """Extract function name near the given line number"""
    lines = content.split('\n')
    
    # Look backwards from the line to find function definition
    for i in range(max(0, line_num - 1), max(0, line_num - 20), -1):
        if i < len(lines):
            line = lines[i]
            # Match Rust function definitions
            fn_match = re.search(r'\b(pub\s+)?(\basync\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)', line)
            if fn_match:
                return fn_match.group(3)
            
            # Match impl blocks
            impl_match = re.search(r'impl\s+(?:.*?\s+for\s+)?([a-zA-Z_][a-zA-Z0-9_]*)', line)
            if impl_match:
                return f"impl {impl_match.group(1)}"
    
    return "unknown"

def find_uncovered_functions(file_analysis: Dict[str, any]) -> List[Tuple[int, str]]:
    """Find functions that have uncovered lines"""
    uncovered_funcs = []
    seen_funcs = set()
    
    for line_num in file_analysis['uncovered_line_numbers']:
        func_name = extract_function_at_line(file_analysis['content'], line_num)
        if func_name not in seen_funcs and func_name != "unknown":
            uncovered_funcs.append((line_num, func_name))
            seen_funcs.add(func_name)
    
    return uncovered_funcs

def print_summary(all_analyses: List[Dict[str, any]]):
    """Print coverage summary"""
    print("=" * 80)
    print("TARPAULIN COVERAGE ANALYSIS")
    print("=" * 80)
    print()
    
    # Overall statistics
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
    print("PER-FILE COVERAGE")
    print("-" * 80)
    print()
    
    # Sort by coverage percentage (lowest first)
    sorted_analyses = sorted(all_analyses, key=lambda x: x['coverage_percent'])
    
    for analysis in sorted_analyses:
        print(f"File: {analysis['path']}")
        print(f"  Coverage: {analysis['coverage_percent']:.2f}%")
        print(f"  Lines: {analysis['covered_lines']}/{analysis['total_lines']}")
        print(f"  Uncovered: {analysis['uncovered_lines']} lines")
        
        # Find uncovered functions
        uncovered_funcs = find_uncovered_functions(analysis)
        if uncovered_funcs:
            print(f"  Uncovered functions/areas:")
            for line_num, func_name in uncovered_funcs[:10]:  # Limit to 10
                print(f"    - {func_name} (near line {line_num})")
        print()

def main():
    # Find the tarpaulin report
    report_file = Path(__file__).parent.parent / "tarpaulin-report.html"
    
    if not report_file.exists():
        print(f"Error: Could not find {report_file}")
        sys.exit(1)
    
    print(f"Analyzing: {report_file}")
    print()
    
    # Extract coverage data
    data = extract_var_data(report_file)
    
    # Analyze each file
    all_analyses = []
    for file_data in data.get('files', []):
        analysis = analyze_file_coverage(file_data)
        all_analyses.append(analysis)
    
    # Print summary
    print_summary(all_analyses)
    
    # Detailed uncovered lines report
    print("=" * 80)
    print("DETAILED UNCOVERED LINES")
    print("=" * 80)
    print()
    
    for analysis in all_analyses:
        if analysis['uncovered_lines'] > 0:
            print(f"\n{analysis['path']}")
            print(f"  Uncovered line numbers: {analysis['uncovered_line_numbers'][:20]}")
            if len(analysis['uncovered_line_numbers']) > 20:
                print(f"  ... and {len(analysis['uncovered_line_numbers']) - 20} more")

if __name__ == "__main__":
    main()
