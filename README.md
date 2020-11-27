# rust-sched
a work in progress scheduled executor, built in rust

## external deps

* cargo make: `cargo install cargo-make`

```
cargo make test
```

## Notes

### Parse incoming OSC expressions?

`x + y` becomes a binding operation with 2 variables.

### Remove Spinlock?

[spinlock considered harmful](https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html)

I used spinlock because I don't want to have system calls in the executing thread.
The idea is that updates will still be evaluated in the executing thread, so there will really never
be any waiting on the spinlock, but there is nothing restricting a user from updating in another thread.

Maybe the executing thread should have some unique access to the resource and other threads just get
some handle?

### Serialize format? 

bindings:
    - id: <uuid>
      alias: <optionalName>
      type: <typename>
      params: #for instance, a cast would have an input format, dest format, binding to cast
        - name: value
        - name: value
        - name: value

nodes: #both graph and non graph nodes??
    - id: <uuid>
      type: <typename>
      alias: <optionalName>
      params:
        - name: value
        - name: value
        - name: value
      children:
        - <uuid>
        - <uuid>
      meta: #optional
        - location: (x, y)
