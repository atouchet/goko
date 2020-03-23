/*
* Licensed to Elasticsearch B.V. under one or more contributor
* license agreements. See the NOTICE file distributed with
* this work for additional information regarding copyright
* ownership. Elasticsearch B.V. licenses this file to you under
* the Apache License, Version 2.0 (the "License"); you may
* not use this file except in compliance with the License.
* You may obtain a copy of the License at
*
*  http://www.apache.org/licenses/LICENSE-2.0
*
* Unless required by applicable law or agreed to in writing,
* software distributed under the License is distributed on an
* "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
* KIND, either express or implied.  See the License for the
* specific language governing permissions and limitations
* under the License.
*/

/*
 * Licensed to Elasticsearch B.V. under one or more contributor
 * license agreements. See the NOTICE file distributed with
 * this work for additional information regarding copyright
 * ownership. Elasticsearch B.V. licenses this file to you under
 * the Apache License, Version 2.0 (the "License"); you may
 * not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *  http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

extern crate protobuf;
extern crate rand;
extern crate yaml_rust;
use std::path::Path;
#[allow(dead_code)]
extern crate grandma;
extern crate pointcloud;
use grandma::utils::*;
use grandma::CoverTreeWriter;
use pointcloud::*;

fn build_tree() -> CoverTreeWriter<L2> {
    let file_name = "../data/mnist_complex.yml";
    let path = Path::new(file_name);
    if !path.exists() {
        panic!(file_name.to_owned() + &" does not exist".to_string());
    }
    cover_tree_from_yaml(&path).unwrap()
}
/*
#[test]
fn load_tree_and_query() {
    let pc = PointCloud::<L2>::from_file(Path::new("data/mnist.yml")).unwrap();
    let zeros = Arc::new(vec![0.0; 784]);
    let ct_loaded = load_tree(Path::new("data/mnist.tree"), pc).unwrap();
    let ct_reader = ct_loaded.reader();
    let query = ct_reader.knn(&zeros, 5).unwrap();
    println!("(array([3.56982747, 3.65066243, 3.83593169, 3.84857365, 3.86859321]), array([17664, 21618, 51468,  8080, 37920]))");
    assert!(query[0].1 == 17664);
    assert!(query[1].1 == 21618);
    assert!(query[2].1 == 51468);
    assert!(query[3].1 == 8080);
    assert!(query[4].1 == 37920);
    assert!(query.len() == 5);
}
*/
//Cover tree on MNIST builds and is queryable
#[test]
fn run_knn_query() {
    let ct = build_tree();
    save_tree(Path::new("../data/mnist.tree"), &ct).unwrap();
    let ct_reader = ct.reader();
    let zeros = [0.0; 784];
    let query = ct_reader.knn(&zeros, 5).unwrap();
    println!("{:#?}", query);
    println!("Expected: (array([3.56982747, 3.65066243, 3.83593169, 3.84857365, 3.86859321]), array([17664, 21618, 51468,  8080, 37920]))");
    assert!(query[0].1 == 17664);
    assert!(query[1].1 == 21618);
    assert!(query[2].1 == 51468);
    assert!(query[3].1 == 8080);
    assert!(query[4].1 == 37920);
    assert!(query.len() == 5);

    // Testing dry insert on prebuilt tree
    let trace = ct_reader.dry_insert(ct_reader.parameters().point_cloud.get_point(59999).unwrap()).unwrap();
    println!("{:?}", trace);
    for t in &trace {
        assert!((t.1).1 == ct_reader.root_address().1);
    }
    let (_dist,last_node_address) = trace.last().unwrap();
    let singleton_condition = ct_reader.get_node_and(*last_node_address,|n| {
        n.singletons().contains(&0)}
    ).unwrap();
    assert!(last_node_address.1 == 59999 || singleton_condition);
}

/*
#[test]
fn run_multiscale_knn_query() {
    let ct = build_tree();
    save_tree(Path::new("../data/mnist.tree"), &ct).unwrap();
    let ct_reader = ct.reader();
    let zeros = Arc::new(vec![0.0; 784]);
    let multiscale_query = ct_reader.multiscale_knn(&zeros, 3).unwrap();

    let dataset = &ct_reader.parameters().point_cloud;
    for (si,layer) in ct_reader.layers() {
        let indexes: Vec<PointIndex> = layer.map_nodes(|si,_| *si);
        let distances = dataset.distances_to_point(&zeros, &indexes).unwrap();
        let mut dist_indexes: Vec<(f32,(i32,PointIndex))> = distances.iter().zip(indexes).map(|(d,pi)| (*d,(si,pi))).collect();
        dist_indexes.sort_by(|(a,_),(b,_)| a.partial_cmp(b).unwrap());
        if dist_indexes.len() > 0 {
            println!("=======");
            if dist_indexes.len() > 3 {
                println!("{:?}", &dist_indexes[0..3]);
            } else {
                println!("{:?}", &dist_indexes);
            }
            println!("{:?}", multiscale_query.get(&si));
        }

    }
    assert!(false);
}
*/
