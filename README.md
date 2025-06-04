# OpenCompute_Worker

The base code for the OpenCompute worker module.

## Prerequisites

- Python 3.10.12+
- Rust (recommended version: 1.86.0 or higher)
- `make`

## Setup and Build

Follow these steps to build and run the project:

### 1. Build the Project
```bash
cd OpenCompute_Worker
make
```

### 2. Start the Server
```bash
cd ./pkg
python -m http.server --bind 0.0.0.0 8000
```

### 3. Access the client
Open the following link in the browser
http://127.0.0.1:8000
