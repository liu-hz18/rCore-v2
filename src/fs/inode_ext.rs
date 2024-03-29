//! 为 [`INode`] 实现 trait [`INodeExt`] 以扩展功能

use super::*;

/// 为 [`INode`] 类型添加的扩展功能
pub trait INodeExt {
    /// 打印当前目录的文件
    fn ls(&self);

    /// 读取文件内容
    fn readall(&self) -> Result<Vec<u8>>;
}

impl INodeExt for dyn INode {
    fn ls(&self) {
        let mut id = 0;
        while let Ok(name) = self.get_entry(id) {
            println!("{}", name);
            id += 1;
        }
    }
    // 从 ELF 文件中加载(读取)用户程序的代码和数据信息，并且映射到内存中。
    fn readall(&self) -> Result<Vec<u8>> {
        // 从文件头读取长度
        let size = self.metadata()?.size;
        // 构建 Vec （映射到内存）并读取
        let mut buffer = Vec::with_capacity(size);
        unsafe { buffer.set_len(size) };
        self.read_at(0, buffer.as_mut_slice())?;
        Ok(buffer)
    }
}
