

struct Board {
    id:u32,
    items:Vec<Rc<dyn BoardItem>>,
}

trait BoardItem {}