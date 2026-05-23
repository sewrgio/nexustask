import subprocess
try:
    result = subprocess.run(['git', 'status'], capture_output=True, text=True)
    print(result.stdout)
    print(result.stderr)
except Exception as e:
    print(e)
