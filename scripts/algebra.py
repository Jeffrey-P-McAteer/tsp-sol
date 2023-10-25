
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
  
def maybe(callback, default_return=None):
  try:
    return callback()
  except:
    traceback.print_exc()
  return default_return

def main(args=sys.argv):
  
  A = Symbol('A')
  B = Symbol('B')
  C = Symbol('C')
  D = Symbol('D')
  E = Symbol('E')
  F = Symbol('F')
  x = Symbol('x')
  y = Symbol('y')

  print(f'solve(x**2 - 1, x) = {maybe(lambda: solve(x**2 - 1, x))}')

  eq = (A*(x**2)) + (B*(x*y)) + (C*(y**2)) + (D*x) + (E*y) + F
  
  print(f'solve(eq, x) = {maybe(lambda: solve(eq, x))}')
  print(f'solve(eq, y) = {maybe(lambda: solve(eq, y))}')
  print(f'solve(eq, A) = {maybe(lambda: solve(eq, A))}')
  print(f'solve(eq, B) = {maybe(lambda: solve(eq, B))}')
  print(f'solve(eq, C) = {maybe(lambda: solve(eq, C))}')
  print(f'solve(eq, D) = {maybe(lambda: solve(eq, D))}')
  print(f'solve(eq, E) = {maybe(lambda: solve(eq, E))}')
  print(f'solve(eq, F) = {maybe(lambda: solve(eq, F))}')

  

  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)



if __name__ == '__main__':
  main()

