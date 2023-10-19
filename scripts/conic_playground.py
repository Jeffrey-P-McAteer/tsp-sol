
import os
import sys
import subprocess
import math
import traceback
import json

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
  a, b, c, d, e, f = coeficients[A], coeficients[B], coeficients[C], coeficients[D], coeficients[E], coeficients[F]
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

  coeficient_min = -50.0
  coeficient_max = 50.0

  graph_x_min = -2.0
  graph_x_max = 16.0
  graph_y_min = -2.0
  graph_y_max = 16.0
  graph_draw_resolution_stop = 0.0004

  graph_x_width = graph_x_max - graph_x_min
  graph_y_height = graph_y_max - graph_y_min

  graph_x0 = tkinter.DoubleVar(value=graph_x_min) # Used to provide run-time changes of x_min and y_min. 
  graph_y0 = tkinter.DoubleVar(value=graph_y_min)

  graph_edge_points = [] # [ [x,y], ] in graph space
  graph_edge_points_file = None
  if len(args) > 1 and os.path.exists(args[1]):
    graph_edge_points_file = args[1]

  print(f'graph_edge_points_file={graph_edge_points_file}')
  if graph_edge_points_file:
    with open(graph_edge_points_file, 'r') as fd:
      graph_edge_points = json.load(fd)

  print(f'graph_edge_points = {graph_edge_points}')

  root.geometry(f'{root_w}x{root_h}')
  root.title('Conic Playground')

  # Useful diagram: https://stackoverflow.com/questions/28089942/difference-between-fill-and-expand-options-for-tkinter-pack-method
  frm = ttk.Frame(root, padding=0)
  frm.pack(fill='both', expand=True)

  canvas = tkinter.Canvas(frm, width=root_w-2, height=root_h-bottom_ui_h, bg='black')
  canvas.pack(fill='both', expand=True)

  controls_frm = ttk.Frame(frm, padding=0)
  controls_frm.pack(side='bottom', fill='x', expand=True)

  right_control_cols = ttk.Frame(controls_frm, padding=0)
  right_control_cols.pack(side='right')

  right_leftmost_rows = ttk.Frame(right_control_cols, padding=5)
  right_leftmost_rows.pack(side='left')

  formula_txt = tkinter.Text(right_leftmost_rows)
  formula_txt.pack(side='top', fill='x', expand=False)

  def reset_all():
    nonlocal last_coef_sum
    a_in.set(1)
    b_in.set(1)
    c_in.set(1)
    d_in.set(1)
    e_in.set(1)
    f_in.set(1)
    x0_in.set(graph_x_min)
    y0_in.set(graph_y_min)
    last_coef_sum = -1.0

  right_rightmost_rows = ttk.Frame(right_control_cols, padding=5)
  right_rightmost_rows.pack(side='right')

  reset_btn = ttk.Button(right_rightmost_rows, text='Reset', command=reset_all)
  reset_btn.pack(side='bottom', fill='both', expand=False)

  q_btn = ttk.Button(right_rightmost_rows, text='Quit', command=root.destroy)
  q_btn.pack(side='top', fill='both', expand=False)

  coeficients = [
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
    tkinter.DoubleVar(value=1.0),
  ]

  currently_redrawing = False

  last_coef_sum = -1.0
  last_draw_resolution = graph_draw_resolution_stop

  def redraw_canvas():
    nonlocal coeficients, canvas, root, currently_redrawing, last_coef_sum, last_draw_resolution
    if currently_redrawing:
      return
    currently_redrawing = True
    try:
      #print(f'coeficients = {coeficients}')
      coeficient_vals = [round(float(x.get()), 2) for x in coeficients]
      addtl_vals = [graph_x0.get(), graph_y0.get()]
      got_new_coefs = abs(sum(coeficient_vals + addtl_vals) - last_coef_sum) > 0.1
      if got_new_coefs:
        last_draw_resolution = 0.5

        # Write formula to text box
        a,b,c,d,e,f = coeficient_vals[A],coeficient_vals[B],coeficient_vals[C],coeficient_vals[D],coeficient_vals[E],coeficient_vals[F],
        formula_str = f'({a}*(x^2)) + ({b}*x*y) + ({c}*(y^2)) + ({d}*x) + ({e}*y) + {f} = 0'
        try:
          formula_txt.delete('1.0', tkinter.END)
        except:
          pass
        formula_txt.insert(tkinter.END, formula_str)

      if last_draw_resolution > graph_draw_resolution_stop:
        # print(f'coeficient_vals = {coeficient_vals}, last_draw_resolution = {last_draw_resolution}')

        last_draw_resolution /= 2.0 # Double resolution (until <= graph_draw_resolution_stop point)

        last_coef_sum = sum(coeficient_vals + addtl_vals)

        canvas_w = float(canvas.winfo_width())
        canvas_h = float(canvas.winfo_height())
        #print(f'canvas = {canvas_w} x {canvas_h}')

        # Clear canvas
        canvas.create_rectangle((0, 0), (canvas_w, canvas_h), fill='black')

        # Draw labels
        x, y = 0, 0
        px_x = ((x - graph_x0.get()) / graph_x_width) * canvas_w
        px_y = ((y - graph_y0.get()) / graph_y_height) * canvas_h
        if px_x > 0 and px_x < canvas_w and px_y > 0 and px_y < canvas_h:
          canvas.create_text(px_x,px_y,fill='grey',font='Arial 12', text='(0,0)')

        # Draw graph_edge_points as red dots
        for point in graph_edge_points:
          if len(point) == 2:
            x, y = point[0], point[1]
            # Transform x and y into screen pixel coords
            px_x = ((x - graph_x0.get()) / graph_x_width) * canvas_w
            px_y = ((y - graph_y0.get()) / graph_y_height) * canvas_h
            canvas.create_rectangle((px_x, px_y), (px_x, px_y), fill='red', outline='')

        # Draw formula
        for x in float_range(graph_x_min, graph_x_max, last_draw_resolution):
          for y in all_conic_y_vals(x, coeficient_vals):
            # Transform x and y into screen pixel coords
            px_x = ((x - graph_x0.get()) / graph_x_width) * canvas_w
            px_y = ((y - graph_y0.get()) / graph_y_height) * canvas_h
            #print(f' {x:.2f}, {y:.2f} screen coords {px_x:.2f}, {px_y:.2f}')
            if px_y > 0.0 and px_y < canvas_h:
              # Paint this pixel white
              canvas.create_rectangle((px_x, px_y), (px_x, px_y), fill='white', outline='')

    except:
      traceback.print_exc()
    currently_redrawing = False
    root.after(50, redraw_canvas) # infinite redraw loop at 20 fps, but we avoid drawing if the sum of coefficients is within 0.01 of the last draw

  sliders_two_col = ttk.Frame(controls_frm, padding=5)
  sliders_two_col.pack(side='left', expand=False)

  sliders_col1 = ttk.Frame(sliders_two_col, padding=5)
  sliders_col1.pack(side='left', expand=False)

  
  a_frm = ttk.Frame(sliders_col1, padding=5)
  a_frm.pack(side='top')
  
  a_label = ttk.Label(a_frm, text="A")
  a_label.pack(side='left')

  a_in = ttk.Scale(a_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[A])
  a_in.set(1)
  a_in.pack(side='right')


  b_frm = ttk.Frame(sliders_col1, padding=5)
  b_frm.pack(side='top')
  
  b_label = ttk.Label(b_frm, text="B")
  b_label.pack(side='left')

  b_in = ttk.Scale(b_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[B])
  b_in.set(1)
  b_in.pack(side='right')



  c_frm = ttk.Frame(sliders_col1, padding=5)
  c_frm.pack(side='top')
  
  c_label = ttk.Label(c_frm, text="C")
  c_label.pack(side='left')

  c_in = ttk.Scale(c_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[C])
  c_in.set(1)
  c_in.pack(side='right')



  x0_frm = ttk.Frame(sliders_col1, padding=5)
  x0_frm.pack(side='top')
  
  x0_label = ttk.Label(x0_frm, text="X0")
  x0_label.pack(side='left')

  x0_in = ttk.Scale(x0_frm, from_=-10.0, to=10.0, orient=HORIZONTAL, length=240, variable=graph_x0)
  x0_in.set(graph_x_min)
  x0_in.pack(side='right')


  sliders_col2 = ttk.Frame(sliders_two_col, padding=5)
  sliders_col2.pack(side='right')


  d_frm = ttk.Frame(sliders_col2, padding=5)
  d_frm.pack(side='top')
  
  d_label = ttk.Label(d_frm, text="D")
  d_label.pack(side='left')

  d_in = ttk.Scale(d_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[D])
  d_in.set(1)
  d_in.pack(side='right')



  e_frm = ttk.Frame(sliders_col2, padding=5)
  e_frm.pack(side='top')
  
  e_label = ttk.Label(e_frm, text="E")
  e_label.pack(side='left')

  e_in = ttk.Scale(e_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[E])
  e_in.set(1)
  e_in.pack(side='right')


  f_frm = ttk.Frame(sliders_col2, padding=5)
  f_frm.pack(side='top')
  
  f_label = ttk.Label(f_frm, text="F")
  f_label.pack(side='left')

  f_in = ttk.Scale(f_frm, from_=coeficient_min, to=coeficient_max, orient=HORIZONTAL, length=300, variable=coeficients[F])
  f_in.set(1)
  f_in.pack(side='right')

  y0_frm = ttk.Frame(sliders_col2, padding=5)
  y0_frm.pack(side='top')
  
  y0_label = ttk.Label(y0_frm, text="Y0")
  y0_label.pack(side='left')

  y0_in = ttk.Scale(y0_frm, from_=-10.0, to=10.0, orient=HORIZONTAL, length=240, variable=graph_y0)
  y0_in.set(graph_y_min)
  y0_in.pack(side='right')



  root.after(150, redraw_canvas)

  float_next_window_async() # Unecessary actually, I've got a rule someplace for tkinter to float
  root.mainloop()

  if 'code' in args or 'i' in args:
    import code
    vars = globals()
    vars.update(locals())
    code.interact(local=vars)

if __name__ == '__main__':
  main()


