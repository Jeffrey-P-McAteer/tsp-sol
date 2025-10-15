
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
    os.makedirs('views', exist_ok=True)

    # maybeset('TSP_INITIAL_COORDS', '3.0,11.0 12.0,11.0 5.0,9.0')
    # maybeset('TSP_ENDING_COORDS', '3.0,11.0 12.0,11.0 7.0,9.0')

    #maybeset('PREF_GPU', 'print')
    maybeset('PREF_GPU', 'none')
    #maybeset('PREF_GPU', 'nvidia')

    maybeset('USE_BRUTE_CACHE', 't')
    #maybeset('USE_BRUTE_CACHE', 'f')

    maybeset('TSP_INITIAL_COORDS', '3.71,8.47 4.66,7.90 7.05,9.73 6.63,9.37 6.54,11.88 3.93,8.23 3.35,3.37 8.73,9.86 ')

    try:
      #subprocess.run('cargo run --release -- multi-pattern-scan 3 0.027 4'.split())

      #subprocess.run('cargo run --release -- selective 9 4'.split())

      subprocess.run('cargo run --release -- spray 8 0.05'.split())

    except:
      if not 'KeyboardInterrupt' in traceback.format_exc():
        traceback.print_exc()



