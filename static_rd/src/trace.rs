use dace::arybase::set_arybase;
use dace::ast::{AryRef, LoopBound, Node, Stmt};
use hist::Hist;
use list_serializable::ListSerializable;
use stack_alg_sim::olken::LRUSplay;
use stack_alg_sim::scale_tree::LRUSplay as LRUScaleTree;
use stack_alg_sim::stack::LRUStack;
use stack_alg_sim::vec::LRUVec;
use stack_alg_sim::LRU;

use std::rc::Rc;

fn access2addr(ary_ref: &AryRef, ivec: &[i32]) -> usize {
    let ary_index = (ary_ref.sub)(ivec);
    if ary_index.len() != ary_ref.dim.len() {
        panic!("array index and dimension do not match");
    }

    let offset = ary_index
        .iter()
        .zip(ary_ref.dim.iter())
        .fold(0, |acc, (&i, &d)| acc * d + i);

    ary_ref.base.unwrap() + offset
}

fn trace_rec_impl(
    code: &Rc<Node>,
    ivec: &mut Vec<i32>,
    sim: &mut Box<dyn LRU<usize>>,
    hist: &mut Hist,
    data_accesses: &mut ListSerializable<usize>,
    dist_rd: &mut ListSerializable<(usize, Option<usize>)>,
) {
    match &code.stmt {
        Stmt::Ref(ary_ref) => {
            let addr = access2addr(ary_ref, ivec);
            data_accesses.add(addr);
            let rd = sim.rec_access(addr);
            dist_rd.add((addr, rd));
            hist.add_dist(rd);
        }
        Stmt::Loop(aloop) => {
            let mut i = match &aloop.lb {
                LoopBound::Fixed(lb) => *lb,
                LoopBound::Dynamic(lb) => lb(ivec),
            };
            let ub = match &aloop.ub {
                LoopBound::Fixed(ub) => *ub,
                LoopBound::Dynamic(ub) => ub(ivec),
            };

            while (aloop.test)(i, ub) {
                ivec.push(i);
                for code in aloop.body.iter() {
                    trace_rec_impl(code, ivec, sim, hist, data_accesses, dist_rd);
                }
                ivec.pop();
                i = (aloop.step)(i);
            }
        }
        Stmt::Block(blk) => blk
            .iter()
            .for_each(|s| trace_rec_impl(s, ivec, sim, hist, data_accesses, dist_rd)),
        Stmt::Branch(stmt) => {
            if (stmt.cond)(ivec) {
                trace_rec_impl(&stmt.then_body, ivec, sim, hist, data_accesses, dist_rd)
            } else if let Some(else_body) = &stmt.else_body {
                trace_rec_impl(else_body, ivec, sim, hist, data_accesses, dist_rd)
            }
        }
    }
}

pub fn trace(
    code: &mut Rc<Node>,
    lru_type: &str,
) -> (
    Hist,
    ListSerializable<(usize, Option<usize>)>,
    ListSerializable<usize>,
) {
    let mut accesses_count: ListSerializable<usize> = ListSerializable::<usize>::new();
    let mut dist_rd: ListSerializable<(usize, Option<usize>)> =
        ListSerializable::<(usize, Option<usize>)>::new();
    let mut hist = Hist::new();
    let split: Vec<&str> = lru_type.split(',').collect();


    let mut analyzer: Box<dyn LRU<usize>> = match split[0] {
        "Olken" => Box::new(LRUSplay::<usize>::new()),
        "Stack" => Box::new(LRUStack::<usize>::new()),
        "Vec" => Box::new(LRUVec::<usize>::new()),
        "Scale" => {
            Box::new(LRUScaleTree::<usize>::new(split[1].parse::<f64>().unwrap(), split[2].parse::<usize>().unwrap()))
        },
        _ => Box::new(LRUSplay::<usize>::new()),
    };

    set_arybase(code);
    println!("{:?}", code);
    trace_rec_impl(
        code,
        &mut Vec::<i32>::new(),
        &mut analyzer,
        &mut hist,
        &mut accesses_count,
        &mut dist_rd,
    );
    (hist, dist_rd, accesses_count)
}

#[cfg(test)]
mod test {
    use super::*;
    use stack_alg_sim::stack::LRUStack;

    #[test]
    fn test_access2addr() {
        let mut aij_node =
            Node::new_ref("x", vec![10, 10], |ij| vec![ij[0] as usize, ij[1] as usize]);
        let mutable = unsafe { Rc::get_mut_unchecked(&mut aij_node) };
        *mutable.ref_only_mut_ref(|a| &mut a.base).unwrap() = Some(0);
        if let Stmt::Ref(aij) = &aij_node.stmt {
            assert_eq!(access2addr(aij, &[0, 0]), 0);
            assert_eq!(access2addr(aij, &[9, 9]), 99);
        }
    }

    #[test]
    fn loop_a_i() {
        // i = 0, 10 { a[i] }
        let mut aref = Node::new_ref("A", vec![10], |i| vec![i[0] as usize]);
        let mut aloop = Node::new_single_loop("i", 0, 10);
        Node::extend_loop_body(&mut aloop, &mut aref);

        let result = trace(&mut aloop, "Stack");
        let hist = result.0;
        assert_eq!(hist.to_vec()[0], (None, 10));
        println!("{}", hist);
    }

    #[test]
    fn loop_a_0() {
        // i = 0, 10 { a[0] }
        let mut aref = Node::new_ref("A", vec![1], |_| vec![0]);
        let mut aloop = Node::new_single_loop("i", 0, 10);
        Node::extend_loop_body(&mut aloop, &mut aref);

        let result = trace(&mut aloop, "Stack");
        let hist = result.0;
        assert_eq!(hist.to_vec()[0], (Some(1), 9));
        assert_eq!(hist.to_vec()[1], (None, 1));
        println!("{}", hist);
    }
}
