## Launcher

Tool for launcher creation from github release.  
You need to create a release on github first; the launcher will take the latest release created.  

### Installation process
1. Clone the repository

```
    git clone https://github.com/2sleepy4u/launcher.git
```

2. Install the tool 

```
    cargo install --path launcher
```

3. Then in the directory where you want to create the executable run

```
    create-launcher [repository]
```

The executable will be created on the same directory you used the tool in.
On windows, if you want, you can place an **icon.ico** inside ./assets directory to give the builded launcer an icon.

#### For example

    create-launcher 2sleepy4u/launcher


