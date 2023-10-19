
import os
import sys
import subprocess
import math
import traceback

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

# Global helpers to index into coeficient tuple
A=0
B=1
C=2
D=3
E=4
F=5

def float_next_window_async():
  subprocess.run(['swaymsg', 'exec', '''sh -c "sleep 0.4 ; swaymsg 'floating enable'" '''])

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
  a = coeficients[A]
  b = coeficients[B]
  c = coeficients[C]
  d = coeficients[D]
  e = coeficients[E]
  f = coeficients[F]
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

def main(args=sys.argv):
  root = Tk()
  root_w = 1400
  root_h = 900
  bottom_ui_h = 150

  coeficient_min = -100.0
  coeficient_max = 100.0

  graph_x_min = -10.0
  graph_x_max = 10.0
  graph_y_min = -10.0
  graph_y_max = 10.0
  graph_draw_resolution = 0.002

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

  coeficients = [
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
  ]

  currently_redrawing = False

  def redraw_canvas():
    nonlocal coeficients, canvas, root, currently_redrawing
    if currently_redrawing:
      return
    currently_redrawing = True
    try:
      #print(f'coeficients = {coeficients}')
      coeficient_vals = [float(x.get()) for x in coeficients]
      print(f'coeficient_vals = {coeficient_vals}')

      canvas_w = float(canvas.winfo_width())
      canvas_h = float(canvas.winfo_height())
      #print(f'canvas = {canvas_w} x {canvas_h}')
      canvas.create_rectangle((0, 0), (canvas_w, canvas_h), fill='black') # clear
      for x in float_range(graph_x_min, graph_x_max, graph_draw_resolution):
        for y in all_conic_y_vals(x, coeficient_vals):
          # Transform x and y into screen pixel coords
          px_x = ((x - graph_x_min) / (graph_x_max - graph_x_min)) * canvas_w
          px_y = ((y - graph_y_min) / (graph_y_max - graph_y_min)) * canvas_h
          #print(f' {x:.2f}, {y:.2f} screen coords {px_x:.2f}, {px_y:.2f}')
          if px_y > 0.0 and px_y < canvas_h:
            # Paint this pixel white
            canvas.create_rectangle((px_x, px_y), (px_x, px_y), fill='white', outline='')


    except:
      traceback.print_exc()
    currently_redrawing = False
    root.after(1000, redraw_canvas) # infinite redraw loop at 1 fps

  sliders_col = ttk.Frame(controls_frm, padding=5)
  sliders_col.pack(side='left')


  a_frm = ttk.Frame(sliders_col, padding=5)
  a_frm.pack(side='bottom')
  
  a_label = ttk.Label(a_frm, text="A")
  a_label.pack(side='left')

  a_in = ttk.Scale(a_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[A])
  a_in.set(1)
  a_in.pack(side='right')



  # b_frm = ttk.Frame(sliders_col, padding=5)
  # b_frm.pack(side='bottom')
  
  # b_label = ttk.Label(b_frm, text="B")
  # b_label.pack(side='left')

  # b_in = ttk.Scale(b_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[B])
  # b_in.set(1)
  # b_in.pack(side='right')



  # c_frm = ttk.Frame(sliders_col, padding=5)
  # c_frm.pack(side='bottom')
  
  # c_label = ttk.Label(c_frm, text="C")
  # c_label.pack(side='left')

  # c_in = ttk.Scale(c_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[C])
  # c_in.set(1)
  # c_in.pack(side='right')


  redraw_canvas()

  float_next_window_async() # Unecessary actually, I've got a rule someplace for tkinter to float
  root.mainloop()

  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)

if __name__ == '__main__':
  main()


