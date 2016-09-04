# Ghostwriter

A git pre-push hook to prevent pushing commits made with a specific author to remote repositories.

## Usage

Add to your git config with the host and allowed author:

```
[ghostwriter "git.bigcorp.com"]
  author = bob@bigcorp.com
[ghostwriter "github.com"]
  author = bob@example.com
```

The above will prevent pushes to `git.bigcorp.com` if any of the commits were not authored by `bob@bigcorp.com`.

You will need to symlink the build artifact into your repositories `.git/hooks` directory as `pre-push`

When pushing you will see something like this:

```
$ git push
verifying commits to git.bigcorp.com are by bob@bigcorp.com
rejecting push due to commits with wrong author:
3b20a8cb7e42d3a08376a7f794eacb5c426ed9bd
error: failed to push some refs to 'https://bob@git.bigcorp.com/bob/WidgetService.git'
```

## Rust

Yes, its probably overkill, but it was more fun than writing a shell script.
