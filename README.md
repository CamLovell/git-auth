# git-auth
A more integrated credential manager for git. It is simple right now but the goal is to remove the need
to think about credentials separately from git itself. Just go use git and if credentials are needed you
will be prompted as required.

git-auth supports multiple accounts and will remember which accounts are associated with which remotes. 
Just `push`, `pull`, `clone`, etc. and the rest will be taken care of.

## Installation
There are no distro-specific packages (yet hopefully).

### Cargo
git-auth is available through [cargo](https://crates.io/crates/git-auth). To install simply run:
```bash
cargo install git-auth
```

Assuming your [path is setup for cargo](https://doc.rust-lang.org/book/ch14-04-installing-binaries.html) installed binaries the command `git-auth` should now be available!

### Build from source
1. Install rust toolchain ([rustup](https://rustup.rs/) is simple but however you would like)
1. Clone the repo and enter
    ```bash
    git clone https://github.com/CamLovell/git-auth
    cd git-auth
    ```
3. Build 
    ```bash
    cargo build --release
    ```

You will now find the binary at `target/release/git-auth`

> [!TIP]
> You can combine the build and setup steps by running `cargo run --release -- init`
> (Note the space before init!)

## Usage
The goal is to be as simple as possible, once installed there is only one manual step:
```bash
git-auth init
```

If you built from source and are in the repository root the command will be something like:
```bash
./target/release/git-auth init
```
(Or refer to the tip above!)

Now just got about your work! When git needs permissions for some remote you will be prompted accordingly,
no need to think about credentials anymore.

If you want to access `git-auth` the initialisation will setup an alias for you to integrate better with git
itself. Use `git auth` (yes, with the space) to access `git-auth` directly.

I'll add more detail here soon, run `git auth -h` for more info.


## Limitations
Very early stages, right now it only supports github over https. Planning to extend this over time.




