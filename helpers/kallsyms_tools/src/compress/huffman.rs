struct Node {
    parent: Option<usize>,
    isright: bool,
    token: Option<usize>,
    score: usize,
}

fn huffman(dic: Vec<(Vec<u8>, usize)>) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut nodes: Vec<Node> = vec![];
    let mut roots: Vec<usize> = vec![];

    for i in 0..dic.len() {
        roots.push(i);
        nodes.push(Node {
            parent: None,
            isright: true, // meaningless if parent is None
            token: Some(i),
            score: dic[i].1,
        })
    }

    while roots.len() >= 2 {
        roots.sort_by(|a, b| nodes[*a].score.cmp(&nodes[*b].score).reverse());

        let scores: Vec<usize> = roots.iter().map(|&i| nodes[i].score).collect();

        let ri = roots.pop().unwrap();
        let li = roots.pop().unwrap();

        let parent = Node {
            parent: None,
            isright: true,
            token: None,
            score: nodes[ri].score + nodes[li].score,
        };
        let parent_i = nodes.len();
        nodes.push(parent);
        roots.push(parent_i);

        nodes[ri].parent = Some(parent_i);
        nodes[ri].isright = true;

        nodes[li].parent = Some(parent_i);
        nodes[li].isright = false;
    }

    let mut table: Vec<(Vec<u8>, Vec<u8>)> = vec![];
    for i in 0..nodes.len() {
        if nodes[i].token.is_none() {
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

        let dic_i = nodes[i].token.unwrap();
        table.push((dic[dic_i].0.clone(), code));
    }

    table
}

#[cfg(test)]
mod tests {
    use crate::compress::huffman::huffman;

    /*
     *                      35
     *                 ______|______
     *                /             \
     *               /               15
     *              /               /  \
     *             20              /    7
     *            /  \            /    / \
     *           11   \          /    /   3
     *          / \    \        /    /   / \
     *        six five nine eight four two one
     *
     *  one    0b1111
     *  two    0b1110
     *  four   0b110
     *  five   0b001
     *  six    0b000
     *  eight  0b10
     *  nine   0b01
     */

    #[test]
    fn test() {
        assert_eq!(
            huffman(vec![
                (b"one".to_vec(), 1),
                (b"two".to_vec(), 2),
                (b"four".to_vec(), 4),
                (b"five".to_vec(), 5),
                (b"six".to_vec(), 6),
                (b"eight".to_vec(), 8),
                (b"nine".to_vec(), 9),
            ]),
            vec![
                (b"one".to_vec(), vec![1, 1, 1, 1]),
                (b"two".to_vec(), vec![1, 1, 1, 0]),
                (b"four".to_vec(), vec![1, 1, 0]),
                (b"five".to_vec(), vec![0, 0, 1]),
                (b"six".to_vec(), vec![0, 0, 0]),
                (b"eight".to_vec(), vec![1, 0]),
                (b"nine".to_vec(), vec![0, 1]),
            ]
        );
    }
}
