use accessibility::{
    AXAttribute, AXUIElement, AXUIElementAttributes, TreeVisitor, TreeWalker, TreeWalkerFlow,
};
use core_foundation::{array::CFArray, string::CFString};
use std::cell::Cell;
use structopt::StructOpt;

struct PrintyBoi {
    level: Cell<usize>,
    max_depth: usize,
    indent: String,
    children: AXAttribute<CFArray<AXUIElement>>,
}

impl PrintyBoi {
    pub fn new_with_indentation(indent: usize, max_depth: usize) -> Self {
        Self {
            level: Cell::new(0),
            max_depth,
            indent: " ".repeat(indent),
            children: AXAttribute::children(),
        }
    }
}

impl TreeVisitor for PrintyBoi {
    fn enter_element(&self, element: &AXUIElement) -> TreeWalkerFlow {
        let indent = self.indent.repeat(self.level.get());
        let role = element.role().unwrap_or_else(|_| CFString::new(""));

        if self.level.replace(self.level.get() + 1) >= self.max_depth {
            return TreeWalkerFlow::SkipSubtree;
        }
        println!(
            "{}- {} ({} children)",
            indent,
            role,
            element.children().map(|a| a.len()).unwrap_or(0)
        );

        if let Ok(names) = element.attribute_names() {
            for name in names.into_iter() {
                if &*name == self.children.as_CFString() {
                    continue;
                }

                if let Ok(value) = element.attribute(&AXAttribute::new(&*name)) {
                    println!["{}|. {}: {:?}", indent, *name, value];
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
    //pub pid: i32,
    pub max_depth: usize,
}

fn main() -> Result<(), i32> {
    let opt = Opt::from_args();
    //let app = AXUIElement::application(opt.pid);
    let app = AXUIElement::system_wide();
    let printy = PrintyBoi::new_with_indentation(4, opt.max_depth);
    let walker = TreeWalker::new();

    walker.walk(&app, &printy);
    Ok(())
}
