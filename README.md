# Counter

## Overview



## Development Workflow

1. Generate program ID and update it in files
```sh
./program_id.sh counter
```

2. Build programs
```sh
./build.sh
```

3. Test programs
* with rebuilding (required if a program was changed)
```sh
./test.sh
```
* w/o rebuilding (faster option)
```sh
./test.sh l
```
