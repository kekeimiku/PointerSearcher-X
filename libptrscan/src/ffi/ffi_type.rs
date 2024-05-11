use core::ffi::c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FFIRange {
    /// 向前偏移
    pub left: usize,
    /// 向后偏移
    pub right: usize,
}

#[repr(C)]
pub struct FFIParam {
    /// 三个必选参数
    /// 目标地址
    pub addr: usize,
    /// 扫描深度/最大指针链长度，极大影响指针链计算速度，越小越快
    pub depth: usize,
    /// 扫描范围，极大影响指针链计算速度，越小越快
    pub srange: FFIRange,

    /// 以下为可选参数
    /// 强制指定最后一级偏移范围，合理使用会极大加快指针链计算速度，默认 `null`,
    /// 与 srange 使用相同的范围
    pub lrange: *const FFIRange,
    /// 限制扫描指针链最短长度，会忽略比 `node`
    /// 短的指针链，几乎不影响指针链计算速度，默认 `null`, 无限制
    /// 如果有值，必须小于或等于 depth
    pub node: *const usize,
    /// 限制指针链必须以指定的 offset 结尾，几乎不影响指针链计算速度，默认
    /// `null`, 无限制
    /// 如果有值，必须在 srange 或 lrange 范围内
    pub last: *const isize,
    /// 限制结果中最大指针链条数，到达最大限制函数会提前返回，默认
    /// `null`，无限制
    pub max: *const usize,
    /// 缩短存在循环引用的指针链，例如指针地址 `P1->P2->P3->P1->P5` 会被缩短成
    /// `P1->P5`, 略微影响指针链计算速度，默认 `false`，不计算循环引用
    pub cycle: bool,

    /// 下面这些参没有意义，用于控制输出原始指针链的格式，暂时没有实现
    pub raw1: bool,
    pub raw2: bool,
    pub raw3: bool,
}

#[repr(C)]
pub struct FFIModule {
    /// 开始地址
    pub start: usize,
    /// 结束地址
    pub end: usize,
    /// 模块路径
    pub pathname: *const c_char,
}
