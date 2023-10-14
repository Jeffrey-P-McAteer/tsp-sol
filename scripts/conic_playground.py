
import os
import sys
import subprocess

try:
  import environmentinator
except:
  env_path = os.path.join(os.getcwd(), '.py-env')
  os.makedirs(env_path, exist_ok=True)
  sys.path.append(env_path)
  subprocess.run([
    sys.executable, '-m', 'pip', 'install', f'--target={env_path}', 'environmentinator'
  ])
  import environmentinator

matplotlib = environmentinator.ensure_module('matplotlib')
numpy = environmentinator.ensure_module('numpy')

import matplotlib.pyplot


def main(args=sys.argv):

  # Hello world from https://stackoverflow.com/questions/30553585/graphing-a-parabola-using-matplotlib-in-python

  # create 1000 equally spaced points between -10 and 10
  x = numpy.linspace(-10, 10, 1000)

  # calculate the y value for each element of the x vector
  y = x**2 + 2*x + 2  

  fig, ax = matplotlib.pyplot.subplots()
  ax.plot(x, y)

  subprocess.run(['swaymsg', 'exec', '''sh -c "sleep 0.4 ; swaymsg 'floating enable'" '''])
  matplotlib.pyplot.show(block=True)

  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)

if __name__ == '__main__':
  main()


