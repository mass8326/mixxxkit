use std::path::{Component, Path, PathBuf};

pub trait ResolveBase {
    fn resolve_base(self, base: impl AsRef<Path>) -> PathBuf;
}

impl ResolveBase for PathBuf {
    fn resolve_base(self, base: impl AsRef<Path>) -> PathBuf {
        match self.is_absolute() {
            true => self,
            false => compress([base.as_ref(), &self].iter().collect::<PathBuf>()),
        }
    }
}

impl ResolveBase for String {
    fn resolve_base(self, base: impl AsRef<Path>) -> PathBuf {
        PathBuf::from(self).resolve_base(base)
    }
}

fn compress(path: impl AsRef<Path>) -> PathBuf {
    let mut components = path.as_ref().components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().copied() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
