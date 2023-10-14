
import os
import sys

try:
  import environmentinator
except:
  import subprocess
  env_path = os.path.join(os.getcwd(), '.py-env')
  os.makedirs(env_path, exist_ok=True)
  sys.path.append(env_path)
  subprocess.run([
    sys.executable, '-m', 'pip', 'install', f'--target={env_path}', 'environmentinator'
  ])
  import environmentinator


def main(args=sys.argv):

  

  import code
  vars = globals()
  vars.update(locals())
  code.interact(local=vars)

if __name__ == '__main__':
  main()


