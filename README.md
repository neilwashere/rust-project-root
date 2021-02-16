# project root

A simple utility to obtain the absolute path to your project root.

## Usage

```rust
let project_root = match project_root::get_project_root() {
    Ok(p) => p.to_str().expect("Could not retrieve project path").to_string(),
    Err(e) => panic!(e),
};
println!("Current project root is {}", project_root);
```

## Motivation

I was trying to slurp in some config files during a test but the directory location
was not what I expected - and indeed would not be the final location of that directory
on a deployment.

I couldn't find an immediately obvious way of finding out my position relative to
the project root so built this little helper.

