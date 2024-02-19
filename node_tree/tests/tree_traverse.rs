use delta::operations::DeltaOperation;
use dom::dom_element::DomElement;
use node_tree::doc_node::DocumentNode;
use node_tree::dom_doc_tree_morph::append;
use node_tree::format_trait::RootFormat;
use node_tree::tree_traverse::{
    first_node, last_block_node, next_node, next_sibling, prev_node, prev_sibling,
};
use op_transform::doc_root::DocumentRoot;
use std::sync::Arc;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

//DONT use a DIV block here, in combination with the used ID this will detect as a root block
static P: &'static str = "P";

fn new_el<'a>(id: String) -> Arc<DocumentNode> {
    let doc_node = DocumentNode::new_element(
        DomElement::new(P),
        Arc::new(RootFormat::new("tree_traverse")),
    );
    doc_node.get_dom_element().unwrap().set_attribute("ID", &id);
    let op = DeltaOperation::insert(id);
    doc_node.set_operation(op);
    Arc::new(doc_node)
}

fn id<'a>(dn: &Arc<DocumentNode>) -> String {
    return dn.get_dom_element().unwrap().get_attribute("id").unwrap();
}

//                A
//            /       \
//           B         J
//         /   \     /   \
//        D    G     K    N   C--- is missing :-)
//       /\    /\    /\   /\
//     E  F   H  I  L M  O  P
//
// Next should return:	-> E,F,D,H,I,G,B,L,M,K,O,P,N,J,A
// Prev should return:  -> A,J,N,P,O,K,M,L,B,G,I,H,D,F,E
#[allow(non_snake_case)]
fn create_test_document(doc: &DocumentRoot) {
    let A = new_el("A".to_string());
    let B = new_el("B".to_string());
    //let C = new_el("C".to_string());
    let D = new_el("D".to_string());
    let E = new_el("E".to_string());
    let F = new_el("F".to_string());
    let G = new_el("G".to_string());
    let H = new_el("H".to_string());
    let I = new_el("I".to_string());
    let J = new_el("J".to_string());
    let K = new_el("K".to_string());
    let L = new_el("L".to_string());
    let M = new_el("M".to_string());
    let N = new_el("N".to_string());
    let O = new_el("O".to_string());
    let Pp = new_el("P".to_string());

    append(&D, E);
    append(&D, F);
    append(&G, H);
    append(&G, I);
    append(&K, L);
    append(&K, M);
    append(&N, O);
    append(&N, Pp);

    append(&B, D);
    append(&B, G);
    append(&J, K);
    append(&J, N);

    append(&A, B);
    append(&A, J);

    append(doc.get_root(), A);
}

//              doc_root
//                |
//                A
//            /       \
//           B         J
//         /   \     /   \
//        D    G     K    N   C--- is missing :-)
//       /\    /\    /\   /\
//     E  F   H  I  L M  O  P
//
// Next should return:	-> E,F,D,H,I,G,B,L,M,K,O,P,N,J,A
#[wasm_bindgen_test]
#[allow(non_snake_case)]
fn next_node_test() {
    let doc = DocumentRoot::new("next_node_test");
    create_test_document(&doc);

    let nxt = first_node(&doc.get_root());
    assert_eq!(id(&nxt), "E");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "F");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "D");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "H");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "I");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "G");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "B");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "L");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "M");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "K");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "O");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "P");
    //error!( "next = {}", id(&nxt) )
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "N");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "J");
    //error!( "next = {}", id(&nxt) );
    let nxt = next_node(&nxt).unwrap();
    assert_eq!(id(&nxt), "A");
    //error!( "next = {}", id(&nxt) );
}

//              doc_root
//                |
//                A
//            /       \
//           B         J
//         /   \     /   \
//        D    G     K    N
//       /\    /\    /\   /\
//     E  F   H  I  L M  O  P
//
// Prev should return:  -> A,J,N,P,O,K,M,L,B,G,I,H,D,F,E
#[wasm_bindgen_test]
#[allow(non_snake_case)]
fn prev_node_test() {
    let doc = DocumentRoot::new("prev_node_test");
    create_test_document(&doc);

    let prev = doc.get_root();
    //error!( "Root = {}", id(&prev) );
    //let prev = prev_node(&prev).unwrap();
    //assert_eq!(id(&prev), "A" );

    if let Some(_) = prev_node(&prev) {
        //No there is no previous from root node
        assert!(false);
    }

    let prev = last_block_node(&doc.get_root()).unwrap();
    assert_eq!(id(&prev), "A");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "J");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "N");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "P");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "O");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "K");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "M");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "L");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "B");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "G");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "I");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "H");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "D");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "F");
    //error!( "Prev = {}", id(&prev) );
    let prev = prev_node(&prev).unwrap();
    assert_eq!(id(&prev), "E");
    //error!( "Prev = {}", id(&prev) );
}

//      A
//     / \
//    B   C
//   / \ / \
//  D  E F  G
// Next should return:	-> D, E, B, F, G, C, A
#[wasm_bindgen_test]
#[allow(non_snake_case)]
fn test_next_sibling_node() {
    let doc = DocumentRoot::new("test_next_node");
    let A = new_el("A".to_string());
    let B = new_el("B".to_string());
    let C = new_el("C".to_string());
    let D = new_el("D".to_string());
    let E = new_el("E".to_string());
    let F = new_el("F".to_string());
    let G = new_el("G".to_string());

    append(&C, F);
    append(&C, G);
    append(&B, D);
    append(&B, E);
    append(&A, B);
    append(&A, C);
    append(doc.get_root(), A);

    let nxt = first_node(&doc.get_root());
    assert_eq!(id(&nxt), "D");
    let nxt = next_sibling(&nxt).unwrap();
    assert_eq!(id(&nxt), "E");
    let nxt = prev_sibling(&nxt).unwrap();
    assert_eq!(id(&nxt), "D");
    let nxt = next_sibling(&nxt).unwrap();
    let nxt = next_sibling(&nxt);
    assert!(nxt == None);
}
