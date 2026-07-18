use accessibility::{
    AXAttribute, AXUIElement, AXUIElementAttributes, TreeVisitor, TreeWalker, TreeWalkerFlow,
};
use objc2_core_foundation::{CFArray, CFString};
use std::cell::Cell;
use structopt::StructOpt;

struct PrintyBoi {
    level: Cell<usize>,
    indent: String,
    children: AXAttribute<CFArray<AXUIElement>>,
}

impl PrintyBoi {
    pub fn new_with_indentation(indent: usize) -> Self {
        Self {
            level: Cell::new(0),
            indent: " ".repeat(indent),
            children: AXAttribute::children(),
        }
    }
}

impl TreeVisitor for PrintyBoi {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        let indent = self.indent.repeat(self.level.get());
        let role = element
            .role()
            .unwrap_or_else(|_| CFString::from_static_str(""));

        self.level.replace(self.level.get() + 1);
        println![
            "{}- {} ({} children)",
            indent,
            role,
            element.children().map(|c| c.len()).unwrap_or_default()
        ];

        if let Ok(names) = element.attribute_names() {
            for name in names.iter() {
                if &*name == self.children.as_CFString() {
                    continue;
                }

                if let Ok(value) = element.attribute(&AXAttribute::new(&name)) {
                    println!["{}|. {}: {:?}", indent, name, value];
                }
            }
        }

        TreeWalkerFlow::Continue
    }

    fn exit_element(&self, _element: &AXUIElement) {
        self.level.replace(self.level.get() - 1);
    }
}

#[derive(StructOpt)]
pub struct Opt {
    pub pid: i32,
}

fn main() -> Result<(), i32> {
    let opt = Opt::from_args();
    let app = AXUIElement::application(opt.pid);
    let printy = PrintyBoi::new_with_indentation(4);
    let walker = TreeWalker::new();

    walker.walk(&app, &printy);
    Ok(())
}
