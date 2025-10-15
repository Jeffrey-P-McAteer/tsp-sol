
# Cross-platform way to track last/recent research tests

import os
import sys
import subprocess
import shutil
import traceback

def maybeset(var_name, value):
  if not var_name in os.environ:
    os.environ[var_name] = value

if __name__ == '__main__':

    if os.path.exists('views'):
        shutil.rmtree('views')

    # maybeset('TSP_INITIAL_COORDS', '3.0,11.0 12.0,11.0 5.0,9.0')
    # maybeset('TSP_ENDING_COORDS', '3.0,11.0 12.0,11.0 7.0,9.0')

    #maybeset('PREF_GPU', 'print')
    maybeset('PREF_GPU', 'none')
    #maybeset('PREF_GPU', 'nvidia')

    maybeset('USE_BRUTE_CACHE', 't')
    #maybeset('USE_BRUTE_CACHE', 'f')

    maybeset('TSP_INITIAL_COORDS', '3.15,5.86 8.09,7.39 10.18,5.37 4.70,6.81 8.09,8.35 11.11,11.05 10.71,11.85 4.80,6.96 9.89,11.42 5.81,6.90 10.26,11.59')

    try:
      #subprocess.run('cargo run --release -- multi-pattern-scan 3 0.027 4'.split())

      #subprocess.run('cargo run --release -- selective 12 4'.split())

      subprocess.run('cargo run --release -- spray 11 0.1'.split())

    except:
      if not 'KeyboardInterrupt' in traceback.format_exc():
        traceback.print_exc()



