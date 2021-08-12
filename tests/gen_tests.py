import random
import sys

with open(f"tests/test_sample{sys.argv[1]}.txt", mode='w') as file:
   for x in range(0, int(sys.argv[2])):
      file.write(str(random.randint(0, 1000)) + ", ")
