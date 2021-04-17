use crate::world::World;

use newport_math::Vector3;

#[cfg(feature = "editable")]
use newport_editor::Editable;

struct Test {
    name: String,
}

#[allow(dead_code)]
#[cfg_attr(feature = "editable", derive(Editable))]
struct Transform {
    position: Vector3,
}

#[test]
fn query() {
    let mut world = World::new();

    world.create()
        .with(Test{
            name: "Hello World".into(),
        })
        .finish();

    let query = world.query()
        .with::<Transform>()
        .with::<Test>()
        .build();

    for e in query.iter() {
        let _test = world.find::<Test>(*e).unwrap();
        let _transform = world.find_mut::<Transform>(*e).unwrap();

        assert!(false);
    }
}

#[test]
fn hello_world() {
    let mut world = World::new();

    let test = world.create()
        .with(Test{
            name: "Hello World".into(),
        })
        .finish();

    let test_struct: &Test = world.find(test).unwrap();

    assert_eq!(test_struct.name, "Hello World");
}