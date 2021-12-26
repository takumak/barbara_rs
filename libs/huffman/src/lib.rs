struct Node {
    parent: Option<usize>,
    isright: bool,
    sym_i: Option<usize>,
    freq: usize,
}

/* input: Vec<(symbol: usize, frequency: usize)>
 * output: Vec<(symbol: usize, frequency: usize, bitarray: Vec<u8>)>
 */
pub fn huffman(symbols: Vec<(usize, usize)>) -> Vec<(usize, usize, Vec<u8>)> {
    let mut nodes: Vec<Node> = vec![];
    let mut roots: Vec<usize> = vec![];

    for sym_i in 0..symbols.len() {
        roots.push(sym_i);
        nodes.push(Node {
            parent: None,
            isright: true, // meaningless if parent is None
            sym_i: Some(sym_i),
            freq: symbols[sym_i].1,
        })
    }

    while roots.len() >= 2 {
        roots.sort_by(|a, b| nodes[*a].freq.cmp(&nodes[*b].freq).reverse());

        let ri = roots.pop().unwrap();
        let li = roots.pop().unwrap();

        let parent = Node {
            parent: None,
            isright: true,
            sym_i: None,
            freq: nodes[ri].freq + nodes[li].freq,
        };
        let parent_i = nodes.len();
        nodes.push(parent);
        roots.push(parent_i);

        nodes[ri].parent = Some(parent_i);
        nodes[ri].isright = true;

        nodes[li].parent = Some(parent_i);
        nodes[li].isright = false;
    }

    let mut table: Vec<(usize, usize, Vec<u8>)> = vec![];
    for i in 0..nodes.len() {
        if nodes[i].sym_i.is_none() {
            continue;
        }

        let mut code: Vec<u8> = vec![];
        let mut j = i;
        loop {
            let node = &nodes[j];
            if node.parent.is_none() {
                break;
            }
            code.insert(0, if node.isright { 1 } else { 0 });
            j = node.parent.unwrap();
        }

        let (sym, freq) = symbols[nodes[i].sym_i.unwrap()];
        table.push((sym, freq, code));
    }

    table
}

#[cfg(test)]
mod tests {
    use crate::huffman;

    /*
     *                    35
     *                  ___|___
     *                 /       \
     *                /         15
     *               /          / \
     *             20          /   7
     *            / \         /   / \
     *           11  \       /   /   3
     *          / \   \     /   /   / \
     *        (6) (5) (9) (8) (4) (2) (1)
     *
     *  (1) 0b1111
     *  (2) 0b1110
     *  (4) 0b110
     *  (5) 0b001
     *  (6) 0b000
     *  (8) 0b10
     *  (9) 0b01
     */

    #[test]
    fn test() {
        assert_eq!(
            huffman(vec![(1, 1), (2, 2), (4, 4), (5, 5), (6, 6), (8, 8), (9, 9),]),
            vec![
                (1, 1, vec![1, 1, 1, 1]),
                (2, 2, vec![1, 1, 1, 0]),
                (4, 4, vec![1, 1, 0]),
                (5, 5, vec![0, 0, 1]),
                (6, 6, vec![0, 0, 0]),
                (8, 8, vec![1, 0]),
                (9, 9, vec![0, 1]),
            ]
        );
    }
}
