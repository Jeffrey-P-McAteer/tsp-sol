
import sys
import os


if __name__ == '__main__':
  coords_s = os.environ.get('TSP_INITIAL_COORDS', None)
  if coords_s is None or len(coords_s) < 2:
    coords_s = sys.argv[1]

  print(f'coords_s={coords_s}')
  print()

  coords_pts = []
  for fragment in coords_s.split():
    try:
      parts = fragment.split(',')
      coords_pts.append(
        (float(parts[0]), float(parts[1]))
      )
    except:
      pass

  print(f'coords_pts={coords_pts}')

  # Should match rust code
  x_min = 3.0
  x_max = 12.0
  y_min = 3.0
  y_max = 12.0
  print(f'rust box dimensions = {x_min},{x_max} x {y_min},{y_max}')
  print()

  coords_x_min = 999999.0
  coords_x_max = -999999.0
  coords_y_min = 999999.0
  coords_y_max = -999999.0
  for x,y in coords_pts:
    if x < coords_x_min:
      coords_x_min = x
    if x > coords_x_max:
      coords_x_max = x

    if y < coords_y_min:
      coords_y_min = y
    if y > coords_y_max:
      coords_y_max = y

  print(f'coords box dimensions = {coords_x_min},{coords_x_max} x {coords_y_min},{coords_y_max}')
  print()

  add_to_x = x_min
  multiply_x_by = (x_max - x_min) / (coords_x_max - coords_x_min)

  add_to_y = y_min
  multiply_y_by = (y_max - y_min) / (coords_y_max - coords_y_min)

  print(f'add_to_x={add_to_x}')
  print(f'multiply_x_by={multiply_x_by}')
  print()

  scaled_coords_pts = []
  for x,y in coords_pts:
    scaled_coords_pts.append(
      (
        (((x - coords_x_min) * multiply_x_by) + add_to_x),
        (((y - coords_y_min) * multiply_y_by) + add_to_y),
      )
    )

  print(f'scaled_coords_pts={scaled_coords_pts}')
  print()

  scaled_coords_s = ' '.join([f'{round(x, 2)},{round(y, 2)}' for x,y in scaled_coords_pts])
  print(f'TSP_INITIAL_COORDS=\'{scaled_coords_s}\'')
  print()





