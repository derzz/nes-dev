#!/usr/bin/env python3
import subprocess
import sys
import re
import signal
import os

# Start the emulator
emulator = subprocess.Popen(["cargo", "run"], stdout=subprocess.PIPE, text=True)

# Read expected output
with open('exact_nocycle.txt', 'r') as f:
    expected_lines = [re.sub(r'PPU:.*$', '', line.rstrip()) for line in f]

# Process output in real-time
line_num = 0
with open('output.txt.part', 'w') as out_file:
    for line in emulator.stdout:
        line = line.rstrip()
        out_file.write(line + '\n')
        
        # Strip PPU info
        stripped_line = re.sub(r'PPU:.*$', '', line)
        
        # Compare with expected line
        if line_num >= len(expected_lines) or stripped_line != expected_lines[line_num]:
            print(f"\n🔴 Difference found at line {line_num+1}:")
            print(f"Expected: '{expected_lines[line_num] if line_num < len(expected_lines) else 'EOF'}'")
            print(f"Got:      '{stripped_line}'")
            
            # Kill the emulator
            emulator.terminate()
            
            # Show context
            print("\n=== Context ===")
            start_line = max(0, line_num - 2)
            end_line = min(len(expected_lines), line_num + 3)
            print(f"Expected (lines {start_line+1}-{end_line}):")
            for i in range(start_line, end_line):
                if i < len(expected_lines):
                    print(f"{i+1}: {expected_lines[i]}")
            print(f"Your output (lines {start_line+1}-{end_line}):")
            out_file.flush()  # <-- Add this line

            with open('output.txt.part', 'r') as partial_out:
                output_lines = [re.sub(r'PPU:.*$', '', l.rstrip()) for l in partial_out]
                for i in range(start_line, end_line):
                    if i < len(output_lines):
                        print(f"{i+1}: {output_lines[i]}")
            # Rename partial output to final output
            os.rename('output.txt.part', 'output.txt')
            sys.exit(1)
        
        line_num += 1

# If we finish without differences, move the output file and report success
os.rename('output.txt.part', 'output.txt')
print("✓ No differences found! Test passed.")
sys.exit(0)
