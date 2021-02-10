# Where you at?

Does your code need an absolute path to the project root?

## Why?

I was trying to slurp in some config files during a test but the directory location
was not what I expected - and indeed would not be the final location of that directory
on a deployment.

I couldn't find an immediately obvious way of finding out my position relative to
the project root so built this little helper.

## Usage

See the example in `lib.rs`
