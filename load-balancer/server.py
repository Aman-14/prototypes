import time

from fastapi import FastAPI

app = FastAPI()


@app.get("/")
async def calculate():
    result = 2 + 2
    # some intense calculation
    time.sleep(10)
    return {"result": result}


if __name__ == "__main__":
    import concurrent.futures
    import subprocess
    import sys

    def run_uvicorn(command):
        process = subprocess.Popen(command)
        process.wait()
        return process.returncode

    with concurrent.futures.ThreadPoolExecutor() as executor:
        # Submit each uvicorn command to the ThreadPoolExecutor
        futures = [
            executor.submit(
                run_uvicorn,
                [
                    sys.executable,
                    "-m",
                    "uvicorn",
                    "server:app",
                    "--host",
                    "0.0.0.0",
                    "--port",
                    f"999{i}",
                ],
            )
            for i in range(1, 6)
        ]

        # Wait for all processes to complete
        concurrent.futures.wait(futures)

        # Access the results if needed
        results = [future.result() for future in futures]
