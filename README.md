# `::named-generics-bundle`

Convenience macros to help with the "bundle multiple generic params with a helper trait" pattern

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/named-generics-bundle.rs)
[![Latest version](https://img.shields.io/crates/v/named-generics-bundle.svg)](
https://crates.io/crates/named-generics-bundle)
[![Documentation](https://docs.rs/named-generics-bundle/badge.svg)](
https://docs.rs/named-generics-bundle)
[![MSRV](https://img.shields.io/badge/MSRV-1.87.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/named-generics-bundle.svg)](
https://github.com/danielhenrymantilla/named-generics-bundle.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/named-generics-bundle.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/named-generics-bundle.rs/actions)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

# API summary

```rust
# use ::named_generics_bundle::named_generics_bundle;
# use Sized as SomeBounds;
#
#[named_generics_bundle]
trait SomeTrait {
    type SomeAssocType: SomeBounds + Clone;
}

fn some_api<S: SomeTrait>(nameable: S::SomeAssocType) {
    // Properly implied bounds.
    nameable.clone();
}

/// We shall have `Example: SomeTrait + Sized + AllStdDeriveTraits`
type Example = SomeTrait![
    # SomeAssocType = (), /*
    SomeAssocType = ...,
    # */
];

some_api::<Example>(
    // …,
    # (),
);
// or, directly:
some_api::<SomeTrait![SomeAssocType = i32]>(42);
```

# Motivation

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

As your Rust projects grows in scope and functionality, your generic types may end up with more and
more generic parameters:

```rust
# use ::core::marker::PhantomData;
# pub trait Burns {}
# enum Uranium {} impl Burns for Uranium {}
# pub trait EnergyForm {}
# enum Beam {} impl EnergyForm for Beam {}
# pub trait YieldsEnergy { fn yield_energy<E: EnergyForm>(&self, _: &mut impl Burns) -> E { todo!() } }
# enum FluxCapacitor {} impl YieldsEnergy for FluxCapacitor {}
#
#[derive(Debug)]
struct Device<Fuel, Engine, Output>
where
    Fuel: Burns,
    Engine: YieldsEnergy,
    Output: EnergyForm,
{
    fuel: Fuel,
    engine: Engine,
    _p: PhantomData<fn() -> Output>,
}

impl<Fuel, Engine, Output> Device<Fuel, Engine, Output>
where
    // 1. First problem: repeat these bounds on every usage of `Device`. So WET… 💦
    Fuel: Burns,
    Engine: YieldsEnergy,
    Output: EnergyForm,
{
    pub fn assemble(fuel: Fuel, engine: Engine) -> Self {
        Self { fuel, engine, _p: PhantomData }
    }

    pub fn frobnicate(&mut self) -> Output {
        self.engine.yield_energy(&mut self.fuel)
    }
}

fn get_away_from_the_beam() -> ! {
    // 2. Second problem: index/position-based generics "tuples"…
    //    😵‍💫 which one was going second again? 😵‍💫
    let mut device = Device::<Uranium, FluxCapacitor, Beam>::assemble(
        // …
        # todo!(), todo!(),
    );
    let beam: Beam = device.frobnicate();
    match beam {}
}
```

It turns out, you can make this already significantly more convenient from the point of view of
the callee by using **a helper trait to _bundle_ all the generic parameters as associated types, and
making the callees generic only over that helper trait**.

```rust
# pub trait Burns {}
# enum Uranium {} impl Burns for Uranium {}
# pub trait EnergyForm {}
# enum Beam {} impl EnergyForm for Beam {}
# pub trait YieldsEnergy { fn yield_energy<E: EnergyForm>(&self, _: &mut impl Burns) -> E { todo!() } }
# enum FluxCapacitor {} impl YieldsEnergy for FluxCapacitor {}
#
pub trait DeviceSetup {
    //! Define your generic bounds *ONCE*. So 🌵 DRY 🐪 🌵
    type Fuel: Burns;
    type Output: EnergyForm;
    type Engine: YieldsEnergy;
}

#[derive(Debug)]
struct Device<S: DeviceSetup> {
    fuel: S::Fuel,
    engine: S::Engine,
}

impl<S: DeviceSetup> Device<S> {
    pub fn assemble(fuel: S::Fuel, engine: S::Engine) -> Self {
        Self { fuel, engine }
    }

    pub fn frobnicate(&mut self) -> S::Output {
        self.engine.yield_energy(&mut self.fuel)
    }
}

fn get_away_from_the_beam() -> ! {
    // No more index/position-based generics "tuples". Get _named_ generic entries.
    // So robust to misordering. Much readability.
    enum BlackMesaStyle {}
    impl DeviceSetup for BlackMesaStyle {
        type Fuel = Uranium;
        type Engine = FluxCapacitor;
        type Output = Beam;
    }
    let mut device = Device::<BlackMesaStyle>::assemble(
        // …
        # todo!(), todo!(),
    );
    let beam: Beam = device.frobnicate();
    match beam {}
}
```

This is already quite a useful trick, but it comes with certain caveats:

  - The most notable one, is that **you can _no longer inline-turbofish_ the generics**: it is
    necessary to define a helper type, and this can get unwieldy when there are outer generic
    parameters in scope.

  - There are also some secondary issues, such as slapping `#[derive(Clone)]` on the `struct Engine`
    and this resulting in `Output` having to be clone, even though it is not part of the actual
    fields of `Engine` (it is a mere dummy `PhantomData` instead).

      - To be fair, this is a limitation/bug stemming from `#[derive(Clone)]` itself, and other
        similar stdlib `#[derive()]`s, being rather dumb w.r.t. the `impl`s they generate, w.r.t.
        adding unnecessary bounds on the params themselves rather than focusing on bounding the
        field types (there is a desire to do the latter, called "perfect derives", but they only
        want to do so once all edge cases, such as recursive types, can be handled. There are also
        some "implicit SemVer" considerations involved as well).

    So you:

      - either make `Output : Clone` even if the only thing susceptible of being `.clone()`d is
        the `Device`,
      - or you stop using `#[derive(Clone)]`, and manually `impl` the trait in question. Which is
        quite cumbersome! Or you involve some third-party lib to help you with that in a smarter or at least more tweakable manner than the stdlib, such as [`::derivative`](
        https://crates.io/crates/derivative) (but this is a rather old crate, nowadays, and most
        people feel like it is unnecessary to add a dependency for such a small thing).

Enters this crate!

</details>

# Detailed Example

```rust
# pub trait Burns {}
# enum Uranium {} impl Burns for Uranium {}
# pub trait EnergyForm {}
# enum Beam {} impl EnergyForm for Beam {}
# pub trait YieldsEnergy { fn yield_energy<E: EnergyForm>(&self, _: &mut impl Burns) -> E { todo!() } }
# enum FluxCapacitor {} impl YieldsEnergy for FluxCapacitor {}
#
#[::named_generics_bundle::named_generics_bundle] // 👈 1. Add this…
pub trait DeviceSetup {
    //! Define your generic bounds *ONCE*. So 🌵 DRY 🐪 🌵
    type Fuel: Burns;
    type Output: EnergyForm;
    type Engine: YieldsEnergy;
}

#[derive(Debug)]
struct Device<S: DeviceSetup> {
    fuel: S::Fuel,
    engine: S::Engine,
}

impl<S: DeviceSetup> Device<S> {
    pub fn assemble(fuel: S::Fuel, engine: S::Engine) -> Self {
        Self { fuel, engine }
    }

    pub fn frobnicate(&mut self) -> S::Output {
        self.engine.yield_energy(&mut self.fuel)
    }
}

fn get_away_from_the_beam() -> ! {
    // No more index/position-based generics "tuples". Get _named_ generic entries.
    // So robust to misordering. Much readability.
    type BlackMesaStyle = DeviceSetup![ // 👈 2. …so you get access to this
        Fuel = Uranium,
        Engine = FluxCapacitor,
        Output = Beam,
    ];
    let mut device = Device::<BlackMesaStyle>::assemble(
        // …
        # todo!(), todo!(),
    );
    let beam: Beam = device.frobnicate();
    match beam {}
}

# struct HandheldPortalDevice {}
# impl YieldsEnergy for HandheldPortalDevice {}
enum Cake {}
# impl EnergyForm for Cake {}
struct TestSubject {}
impl Burns for TestSubject {}

fn the_cake_is_a_lie() -> ! {
    let testing_chamber =
        // 👇 3. It can also be turbofished-inlined!
        Device::<DeviceSetup![
            Engine = HandheldPortalDevice,
            Output = Cake,
            Fuel = TestSubject,
        ]>::assemble(
            // …
            # todo!(), todo!(),
        )
    ;
    let puzzle_solved: Cake = testing_chamber.frobnicate();
    match puzzle_solved {}
}
```

---

See the docs of [`#[named_generics_bundle]`][`named_generics_bundle`] for more info.

[`named_generics_bundle`]: https://docs.rs/named-generics-bundle/*/named_generics_bundle/attr.named_generics_bundle.html
