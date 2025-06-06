
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

    maybeset('TSP_INITIAL_COORDS', '3.0,11.0 12.0,11.0 5.0,9.0')
    maybeset('TSP_ENDING_COORDS', '3.0,11.0 12.0,11.0 7.0,9.0')
    #maybeset('PREF_GPU', 'print')
    #maybeset('PREF_GPU', 'none')
    maybeset('PREF_GPU', 'nvidia')
    maybeset('USE_BRUTE_CACHE', 't')

    try:
      subprocess.run('cargo run --release -- multi-pattern-scan 3 0.027 4'.split())
    except:
      if not 'KeyboardInterrupt' in traceback.format_exc():
        traceback.print_exc()



