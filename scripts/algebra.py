
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

def main(args=sys.argv):
  
  A = Symbol('A')
  B = Symbol('B')
  C = Symbol('C')
  D = Symbol('D')
  E = Symbol('E')
  F = Symbol('F')
  x = Symbol('x')
  y = Symbol('y')
  v = Symbol('v') # used as an equality "Value" for the right-hand-side

  print(f'solve(x**2 - 1, x) = {maybe(lambda: solve(x**2 - 1, x))}')

  eq = Eq(v, (A*(x**2)) + (B*(x*y)) + (C*(y**2)) + (D*x) + (E*y) + F).subs(v, 0)

  
  print(f'solve(eq, x) = {maybe(lambda: solve(eq, x))}')
  print(f'solve(eq, y) = {maybe(lambda: solve(eq, y))}')
  print(f'solve(eq, A) = {maybe(lambda: solve(eq, A))}')
  print(f'solve(eq, B) = {maybe(lambda: solve(eq, B))}')
  print(f'solve(eq, C) = {maybe(lambda: solve(eq, C))}')
  print(f'solve(eq, D) = {maybe(lambda: solve(eq, D))}')
  print(f'solve(eq, E) = {maybe(lambda: solve(eq, E))}')
  print(f'solve(eq, F) = {maybe(lambda: solve(eq, F))}')
  print()

#   for x_val in float_range(-5.0, 5.0, 0.25):
#     # parabola: Ax^2 + Dx + Ey = 0
#     eq_2 = Eq(v, (A*(x_val**2)) + (B*(x_val*y)) + (C*(y**2)) + (D*x_val) + (E*y) + F).subs(v, 0)
#     eq_3 = Eq(v, (A*(x_val**2)) + (D*x_val) + (E*y) ).subs(v, 0)

#     eqs = [
#       eq_2, eq_3
#     ]

#     print(f'solve(eqs, y) = {maybe(lambda: solve(eqs, y))}')
#     print(f'solve(eqs, A) = {maybe(lambda: solve(eqs, A))}')
#     print(f'solve(eqs, B) = {maybe(lambda: solve(eqs, B))}')
#     print(f'solve(eqs, C) = {maybe(lambda: solve(eqs, C))}')
#     print(f'solve(eqs, D) = {maybe(lambda: solve(eqs, D))}')
#     print(f'solve(eqs, E) = {maybe(lambda: solve(eqs, E))}')
#     print(f'solve(eqs, F) = {maybe(lambda: solve(eqs, F))}')
#     print()

  equations = [
    Eq(0, (A*(x**2)) + (B*(x*y)) + (C*(y**2)) + (D*x) + (E*y) + F),
    Eq(0, (A*(x**2)) + (D*x) + (E*y) ),
  ]
#   known_xys = [
#     # (1.0, 1.0),
#     # #(1.0, -1.0),
#     # (0.0, 0.0),
#     # (-1.0, 1.0),
#     # #(-1.0, -1.0),
#     (-2.0, (-2.0)**2),
#     (-1.0, (-1.0)**2),
#     (0.0, 0.0),
#     (1.0, (1.0)**2),
#     (2.0, (2.0)**2),
#   ]
#   for x_val,y_val in known_xys:
#     equations.append(
#       Eq(v, (A*(x**2)) + (B*(x*y)) + (C*(y**2)) + (D*x) + (E*y) + F)
#          .subs(v, 0)
#          .subs(x, x_val)
#          .subs(y, y_val)
#     )

  for e in equations:
      print(f'e >> {e}')

  print(f'solve(equations, x) = {maybe(lambda: solve(equations, x))}')
  print(f'solve(equations, y) = {maybe(lambda: solve(equations, y))}')
  print(f'solve(equations, A) = {maybe(lambda: solve(equations, A))}')
  print(f'solve(equations, B) = {maybe(lambda: solve(equations, B))}')
  print(f'solve(equations, C) = {maybe(lambda: solve(equations, C))}')
  print(f'solve(equations, D) = {maybe(lambda: solve(equations, D))}')
  print(f'solve(equations, E) = {maybe(lambda: solve(equations, E))}')
  print(f'solve(equations, F) = {maybe(lambda: solve(equations, F))}')
  print()


  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)



if __name__ == '__main__':
  main()
