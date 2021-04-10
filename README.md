# project root

A simple utility to obtain the absolute path to your project root.

## Usage

```rust
match project_root::get_project_root() {
    Ok(p) => println!("Current project root is {:?}", p),
    Err(e) => println!("Error obtaining project root {:?}", e)
};
```

## Motivation

I was trying to slurp in some config files during a test but the directory location
was not what I expected - and indeed would not be the final location of that directory
on a deployment.

I couldn't find an immediately obvious way of finding out my position relative to
the project root so built this little helper.


## Usage

See the example in `lib.rs`

Just a commit to see what a diff might look like
