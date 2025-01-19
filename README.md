# Post-Archiver-Viewer

To view `Post Archiver`  

## Preview
Author List
![preview](preview.png)
Post
![preview](preview-1.png)

## Debug or Build
Frontend
```sh
cd frontend;
bun install;
bun run dev;
```
Backend
```sh
# dev profile will proxy 5137 (vite server for HMR)
cargo run
```

## Deploy
```sh
cd frontend;
bun install;
bun run build;
cd ..;
cargo build -r;
```