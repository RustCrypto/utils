# TODO:

* Add support for async closures, possibly using a macro to define the functions if necessary. Use `futures::executor::block_on(f())` to poll the entire future completion inside the stack switched context, and avoid `.await` that yields control outside of the `on_stack()` boundary. Something like:

```rust
pub unsafe fn exec_async_on_sanitized_stack<Fut, F, R>(
    stack: &mut [u8],
    f: F,
) -> Result<R, Box<dyn std::any::Any + Send>>
where
    F: FnOnce() -> Fut + UnwindSafe,
    Fut: Future<Output = R>,
{
    let mut result = None;

    on_stack(stack, || {
        result = Some(catch_unwind(AssertUnwindSafe(|| {
            // Block on the future inside the heap stack
            futures::executor::block_on(f())
        })));
    });

    result.expect("Closure did not run")
}
```

* Handle unwinds better: currently we return a `Result<R, Box<dyn Any + Send>>`. The error case is a little bit tricky to handle, as dropping the error could cause a panic. The program should either panic, or return the panic payload's message.

* Use stacker crate to handle stack size management: if I read some of the `stacker` docs correctly, that crate should be able to extend the size of the stack when it is about to overflow. If that is correct, we could use their techniques to allocate a new stack and zeroize the old one whenever our allocated stack is about to overflow, eliminating the primary remaining `# Safety` comment. Note: we may not be able to zeroize the old stack immediately as the stack switching process likely attempts to return to the old stack once execution completes; we might have to wait until execution completes before zeroizing all heap-stacks.

* Add an `asm!` alternative method for stack bleaching. In theory, it would be better to use `asm!` as we would not need to worry about the size of the allocated switched stack, and it would keep all of the code running on the actual stack and not the heap, possibly preserving performance. The problem with this is that using pointers from `asm!` and rust code to zero the space between the pointers results in segmentation faults on `x86_64`.
  * when testing this, assert that the two pointers are not equal to each other and not null.