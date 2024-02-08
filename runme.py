
# Cross-platform way to track last/recent research tests

import os
import sys
import subprocess
import shutil

if __name__ == '__main__':
    
    if os.path.exists('views'):
        shutil.rmtree('views')

    os.environ['TSP_INITIAL_COORDS'] = '3.0,11.0 12.0,11.0 5.0,9.0'
    os.environ['TSP_ENDING_COORDS'] = '3.0,11.0 12.0,11.0 7.0,9.0'
    os.environ['PREF_GPU'] = 'print'

    subprocess.run('cargo run --release -- multi-pattern-scan 3 0.027 4'.split())



