<EstimatedTime>
	<span>READING</span>
	<SmallBullet>•</SmallBullet>
	<span>3 MIN</span>
</EstimatedTime>

# Using

---

The `using` keyword brings children (properties, methods, etc.) of an object into scope by their name, or an optional alias. This is usually done to simplify the usage of namespace functions or to properties of something to the local scope.

<br>

Write `using` followed by the object and dot/double-colon notation followed by the property you want to expose.

```ignite
let my_user = {
	name = "JohnDoe",
	age = 18
}

using my_user.name
// or
using my_user::name

// `name` is a now a variable
println(name)
```

<br>

**Note:** The `using` keyword is local scope, which means it cannot be accessed globally.

<br>

Here's an example of exposing the `abs` function from the `Math` standard library:

```ignite
using Std::Math::abs

println(abs(-50)) // prints 50
```

You can even import namespaces directly so you don't have to prefix them with Std:

```ignite
using Std::Math

println(Math.clamp(17, 0, 10)) // prints 10
```

## Exposing Multiple Properties

Use curly braces (`{}`) into the using syntax to expose multiple properties/methods at once.

```ignite
using Std::Math::{sin, PI}

println(sin(PI / 2)) // prints 1
```

```ignite
using Std::{Math, Random, Http}

println(Math.sin(Math.PI / 2)) // prints 1
println(Random.float_range(0, 10))
```

## Wildcard Imports

Import everything from a **namespace** into the current scope. This is generally not recommended since it can pollute your scope with stuff or override previous variables if you aren't careful.

<br>

Use an asterisk (`*`) to import everything from a namespace.

```ignite
using Std::Math::*

print(abs(-20))
print(round(1.5))
```
