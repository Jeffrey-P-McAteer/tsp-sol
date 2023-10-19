
import os
import sys
import subprocess

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

# matplotlib = environmentinator.ensure_module('matplotlib')

def float_next_window_async():
  subprocess.run(['swaymsg', 'exec', '''sh -c "sleep 0.4 ; swaymsg 'floating enable'" '''])


def main(args=sys.argv):
  root = Tk()
  root_w = 1400
  root_h = 900
  bottom_ui_h = 150
  root.geometry(f'{root_w}x{root_h}')
  root.title('Conic Playground')

  # Useful diagram: https://stackoverflow.com/questions/28089942/difference-between-fill-and-expand-options-for-tkinter-pack-method
  frm = ttk.Frame(root, padding=0)
  frm.pack(fill='both', expand=True)

  canvas = tkinter.Canvas(frm, width=root_w-2, height=root_h-bottom_ui_h, bg='black')
  canvas.pack(fill='both', expand=True)

  controls_frm = ttk.Frame(frm, padding=0)
  controls_frm.pack(side='bottom', fill='x', expand=True)

  q_btn = ttk.Button(controls_frm, text='Quit', command=root.destroy)
  q_btn.pack(side='right', fill='both', expand=False)

  a_label = Label(controls_frm, text="a")
  a_label.pack(side='left')
  
  a_in = ttk.Scale(controls_frm, from_=8, to=50, orient=HORIZONTAL, length=300)
  a_in.set(30)
  a_in.pack(side='left')

  float_next_window_async() # Unecessary actually, I've got a rule someplace for tkinter to float
  root.mainloop()

  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)

if __name__ == '__main__':
  main()


