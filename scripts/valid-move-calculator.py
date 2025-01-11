
import os
import sys
import subprocess

py_env = os.path.join(os.path.dirname(__file__), '.py-env')
os.makedirs(py_env, exist_ok=True)
sys.path.append(py_env)

def pip_import(module_name, pkg_name=None):
  if pkg_name is None:
    pkg_name = module_name
  if not module_name in globals() or globals()[module_name] is None:
    try:
      globals()[module_name] = __import__(module_name)
    except:
      subprocess.run([
        sys.executable, '-m', 'pip', 'install', f'--target={py_env}', pkg_name
      ])
      globals()[module_name] = __import__(module_name)

pip_import('tkinter', 'tk')

def main(args=sys.argv):
  print(f'tkinter={tkinter}')

  class Application(tkinter.Frame):
    def __init__(self, master=None):
      tkinter.Frame.__init__(self, master)
      self.grid()
      self.draw_once()

    def draw_once(self):
      self.update()

      for child in self.winfo_children():
        child.destroy()

      print(f'{self.master.winfo_width()}')
      self.can = tkinter.Canvas(self.master, width=self.master.winfo_width(), height=300)
      self.can.grid(row=2, column=1)
      self.can.create_line(0,0,500,200)

      self.after(150, self.draw_once)



  root = tkinter.Tk()
  app = Application(master=root)
  app.mainloop()
  root.destroy()




if __name__ == '__main__':
  main()
