pub mod marker;
pub mod marker_item_link;
pub mod marker_linkage;
/// 打点提交审批表。现已正式启用——实现了完整的暂存/审核中/不通过状态机，
/// 以及通过审批后的 method_type 晋升（Added→插入 marker，Modified→更新，Deleted→软删除）。
pub mod marker_punctuate;
