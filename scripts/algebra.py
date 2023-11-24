
import os
import sys
import subprocess
import math
import traceback
import json
import re
import shutil

# Ought to come w/ python, if not see your OS's package manager for a copy pf python-tkinter
import tkinter
from tkinter import *
from tkinter import ttk

env_path = os.path.join(os.getcwd(), '.py-env')
os.makedirs(env_path, exist_ok=True)
sys.path.append(env_path)

try:
  import environmentinator
except:
  subprocess.run([
    sys.executable, '-m', 'pip', 'install', f'--target={env_path}', 'environmentinator'
  ])
  import environmentinator

sympy = environmentinator.ensure_module('sympy')
from sympy.solvers import solve
from sympy import Symbol
from sympy import Eq
from sympy import sin, cos

def maybe(callback, default_return=None):
  try:
    return callback()
  except:
    traceback.print_exc()
  return default_return

def float_range(begin_f, end_f, step_f):
  if begin_f < end_f and step_f < 0.0:
    raise Exception('Infinite float_range!')
  elif begin_f > end_f and step_f > 0.0:
    raise Exception('Infinite float_range!')

  val = begin_f
  while val < end_f:
    yield val
    val += step_f
  yield end_f

def all_conic_y_vals(x, coeficients):
  a, b, c, d, e, f = coeficients[0], coeficients[1], coeficients[2], coeficients[3], coeficients[4], coeficients[5]
  if c != 0:
    try:
      yield -(
            math.sqrt( ((b*x) + e)**2.0 - (4.0*c*( (x*((a*x) + d)) + f)) ) + (b*x) + e
          ) / (
              2.0*c
        )
    except:
      #traceback.print_exc()
      pass
    try:
      yield  -(
          -(math.sqrt( ((b*x) + e)**2.0 - (4.0*c*( (x*((a*x) + d)) + f)) )) + (b*x) + e
          ) / (
              2.0*c
        )
    except:
      #traceback.print_exc()
      pass

  if c == 0.0 and ((b*x)+e) != 0.0:
    try:
      yield -( (x*( (a*x) + d )) + f ) / ( (b*x) + e )
    except:
      #traceback.print_exc()
      pass

def dump_c(msg, caret, possible_list):
  print(msg)
  if possible_list is None:
    possible_list = []
  for item in possible_list:
    print(f'{caret}> {item}')

def dump(msg, possible_list):
  print(msg)
  if possible_list is None:
    possible_list = []
  for item in possible_list:
    print(f' > {item}')


def main(args=sys.argv):

  A = Symbol('A')
  B = Symbol('B')
  C = Symbol('C')
  D = Symbol('D')
  E = Symbol('E')
  F = Symbol('F')

  R = Symbol('R') # Roll
  P = Symbol('P') # Pitch

  x = Symbol('x')
  y = Symbol('y')
  z = Symbol('z')
  v = Symbol('v') # used as an equality "Value" for the right-hand-side

  eq = Eq(0, (A*(x**2)) + (B*(x*y)) + (C*(y**2)) + (D*x) + (E*y) + F)

  equations = [
    Eq(z**2, (((x*cos(P))-(y*sin(P)))**2) - (((x*sin(P)) + (y*cos(P)))**2)  ),
    Eq(x**2, (((z*cos(R))-(y*sin(R)))**2) - (((z*sin(R)) + (y*cos(R)))**2)  ),
  ]

  dump_c('Input Equations: ', ' e', equations)

  dump(f'solve(equations, x) = ', maybe(lambda: solve(equations, x)))
  dump(f'solve(equations, y) = ', maybe(lambda: solve(equations, y)))
  dump(f'solve(equations, z) = ', maybe(lambda: solve(equations, z)))
  print()

  # dump(f'solve(equations, (A,B,C,D,E,F)) = ', maybe(lambda: solve(equations, (A,B,C,D,E,F) )))

  print()


  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)



if __name__ == '__main__':
  main()

