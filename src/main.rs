use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use sv_parser::{parse_sv, unwrap_locate, unwrap_node, Locate, PortDeclaration, RefNode, SyntaxTree};


fn main() {
    // 1. 从命令行参数获取 Verilog 文件路径
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     eprintln!("错误: 请提供 Verilog 文件的路径。");
    //     eprintln!("用法: cargo run -- <file_path.v>");
    //     return;
    // }
    let path = PathBuf::from("./test/std-7.1.6-primitives.v");
    // if !path.exists() {
    //     eprintln!("错误: 文件 '{}' 不存在。", path.display());
    //     return;
    // }

    let mut file = File::create("dump").unwrap();


    // 2. 设置空的宏定义和包含路径
    let defines = HashMap::new();
    let includes: Vec<PathBuf> = Vec::new();

    // 3. 使用 sv-parser 解析文件
    match parse_sv(&path, &defines, &includes, false, false) {
        Ok((syntax_tree, _)) => {
            println!("文件 '{}' 解析成功！开始提取信息...", path.display());
            // println!("module is :{}", syntax_tree);
            writeln!(file, "module is {}", syntax_tree).unwrap();
            // 4. 遍历语法树的顶层节点
            for node in &syntax_tree {
                match node {
                    // 匹配 ANSI 风格的模块声明 `module my_module (...)`
                    RefNode::ModuleDeclarationAnsi(x) => {
                        println!("\n--------------------------------------------------");
                        extract_module_info(&RefNode::from(x), &syntax_tree, true);
                    }
                    // 匹配非 ANSI 风格的模块声明 `module my_module (a, b, c)`
                    RefNode::ModuleDeclarationNonansi(x) => {
                        println!("\n--------------------------------------------------");
                        extract_module_info(&RefNode::from(x), &syntax_tree, false);
                    }
                    _ => (),
                }
            }
        }
        Err(e) => {
            eprintln!("文件解析失败: {:?}", e);
        }
    }
}

// 提取并打印模块信息的主函数
fn extract_module_info(node: &RefNode, syntax_tree: &SyntaxTree, is_ansi: bool) {
    // 提取模块名
    if let Some(module_identifier) = unwrap_node!(node.clone(), ModuleIdentifier) {
        let module_name = get_identifier_string(module_identifier, syntax_tree).unwrap_or_default();
        let style = if is_ansi { "ANSI" } else { "Non-ANSI" };
        println!("模块名: {} (风格: {})", module_name, style);

        // 提取端口信息
        println!("\n  端口信息:");
        if is_ansi {
            extract_ansi_ports(node, syntax_tree);
        } else {
            extract_nonansi_ports(node, syntax_tree);
        }

        // 提取例化信息
        println!("\n  例化信息:");
        // extract_instantiation_info(node, syntax_tree);
    }
    // println!("module is :{}", syntax_tree);
}

// 提取 ANSI 风格模块的端口信息
fn extract_ansi_ports(node: &RefNode, syntax_tree: &SyntaxTree) {
    // 遍历所有 AnsiPortDeclaration 节点
    for port_declaration in unwrap_node!(node.clone(), AnsiPortDeclaration).into_iter().flatten() {
        let mut direction_str = "inout".to_string(); // 默认方向
        let mut width_str = "1-bit".to_string(); // 默认位宽
        let mut port_name = "Unknown-name";

        match unwrap_node!(port_declaration, PortDirection, PackedDimension, PortIdentifier) {
            None => {}
            Some(RefNode::PortDeclaration(port_direction)) => {
                direction_str = direction_to_str(port_direction);
            },
            Some(RefNode::PackedDimension(packed_dimension)) => {
                width_str = get_node_string(RefNode::from(packed_dimension), syntax_tree);
            },
            Some(RefNode::PortIdentifier(port_identifier)) => {
                port_name = get_identifier_string(RefNode::from(port_identifier), syntax_tree).unwrap();
            },
            _ => {}
        }
        println!("    - 名称: {:<20} 方向: {:<10} 位宽: {:<10}", port_name, direction_str, width_str);

    }
}

// 提取非 ANSI 风格模块的端口信息
fn extract_nonansi_ports(node: &RefNode, syntax_tree: &SyntaxTree) {
    // 非 ANSI 风格的端口信息分散在两处：
    // 1. 模块头部的 `ListOfPorts`：只包含端口名称。
    // 2. 模块体内的 `PortDeclaration`：包含方向、位宽和名称。
    // 我们主要从 `PortDeclaration` 中提取完整信息。

    // 遍历模块体内的所有 `ModuleItem`
    for item in unwrap_node!(node.clone(), ModuleItem).into_iter().flatten() {
        // 找到端口声明节点
        // if let Some(port_declaration) = unwrap_node!(item, PortDeclaration) {
            let mut direction_str = "None-dir".to_string();
            let mut width_str = "None-width".to_string();
            let mut port_name = "None-name";

            println!("phase 1 start --------------------------- cur node {}", item);

            // for port in unwrap_node!(item, PortDeclaration, PackedDimension, ListOfPortIdentifiers).into_iter().flatten() {
            //     match port {
            //         RefNode::PortDeclaration(port_dir) => {
            //             direction_str = direction_to_str(port_dir);
            //             println!("update port direction");
            //         },
            //         RefNode::PackedDimension(port_dimension) => {
            //             println!("port {} 's dimension is : {}", direction_str, RefNode::from(port_dimension));
            //         }
            //         _ => {}
            //     }
            // }

            // match unwrap_node!(item, PortDeclaration, PackedDimension, ListOfPortIdentifiers) {
            //     None => {}
            //     Some(RefNode::PortDeclaration(port_direction)) => {
            //         direction_str = direction_to_str(port_direction);
            //     },
            //     Some(RefNode::PackedDimension(packed_dimension)) => {
            //         width_str = get_node_string(RefNode::from(packed_dimension), syntax_tree);
            //     },
            //     Some(RefNode::ListOfPortIdentifiers(list_of_port_identifiers)) => {
            //         for port_identifier in unwrap_node!(list_of_port_identifiers, PortIdentifier).into_iter().flatten() {
            //             port_name = get_identifier_string(port_identifier, syntax_tree).unwrap_or("None name");
            //
            //         }
            //     },
            //     _ => {}
            // }
            // println!("    - 名称: {:<20} 方向: {:<10} 位宽: {:<10}", port_name, direction_str, width_str);

            // // 提取该声明下的所有端口名
            // if let Some(list_of_port_identifiers) = unwrap_node!(port_declaration, ListOfPortIdentifiers) {
            //     for port_identifier in unwrap_node!(list_of_port_identifiers, PortIdentifier).into_iter().flatten() {
            //         let port_name = get_identifier_string(port_identifier, syntax_tree).unwrap_or(&"未知端口".to_string());
            //         println!("    - 名称: {:<20} 方向: {:<10} 位宽: {:<10}", port_name, direction_str, width_str);
            //     }
            // }
        // }
    }
    println!("Module item extract end ************")
}


// // 提取例化信息
// fn extract_instantiation_info(node: &RefNode, syntax_tree: &SyntaxTree) {
//     // 遍历模块体内的所有 `ModuleItem`
//     for item in unwrap_node!(node, ModuleItem).into_iter().flatten() {
//         // 找到模块例化节点
//         if let Some(instantiation) = unwrap_node!(item, ModuleInstantiation) {
//             // 提取被例化的模块名
//             let module_name = unwrap_node!(instantiation, ModuleIdentifier)
//                 .and_then(|id| get_identifier_string(id, syntax_tree))
//                 .unwrap_or_else(|| "未知模块".to_string());
//
//             // 遍历该模块的所有例化实例 (例如 `sub u1(...)`, `u2(...)`)
//             for instance in unwrap_node!(instantiation, HierarchicalInstance).into_iter().flatten() {
//                 // 提取例化名 (例如 `u1`)
//                 let instance_name = unwrap_node!(instance, NameOfInstance)
//                     .and_then(|noi| unwrap_node!(noi, InstanceIdentifier))
//                     .and_then(|id| get_identifier_string(id, syntax_tree))
//                     .unwrap_or_else(|| "未命名例化".to_string());
//
//                 println!("    - 例化模块: {:<20} 例化名: {}", module_name, instance_name);
//
//                 // 提取端口连接信息
//                 if let Some(list_of_port_connections) = unwrap_node!(instance, ListOfPortConnections) {
//                     for named_port_connection in unwrap_node!(list_of_port_connections, NamedPortConnection).into_iter().flatten() {
//                         let port_name = unwrap_node!(named_port_connection, PortIdentifier)
//                             .and_then(|id| get_identifier_string(id, syntax_tree))
//                             .unwrap_or("".to_string());
//
//                         // 连接到端口的信号名
//                         let connected_signal = unwrap_node!(named_port_connection, Expression)
//                             .map(|expr| get_node_string(expr, syntax_tree))
//                             .unwrap_or_else(|| "未连接".to_string());
//
//                         println!("        - 端口: {:<20} -> 信号: {}", port_name, connected_signal);
//                     }
//                 }
//             }
//         }
//     }
// }

// 从 Identifier 类节点安全地获取其字符串表示
// fn get_identifier_string(identifier: RefNode, syntax_tree: &SyntaxTree) -> Option<String> {
//     unwrap_node!(identifier, Identifier)
//         .and_then(|node| unwrap_locate!(node))
//         .and_then(|locate| syntax_tree.get_str(locate))
//         .map(|s| s.to_string())
// }

// 从任意节点获取其原始的字符串表示
fn get_node_string(node: RefNode, syntax_tree: &SyntaxTree) -> String {
    unwrap_locate!(node)
        .and_then(|locate| syntax_tree.get_str(locate))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn direction_to_str(dir: &PortDeclaration) -> String {
    match dir {
        PortDeclaration::Inout(_) => "Inout".into(),
        PortDeclaration::Input(_) => "Input".into(),
        PortDeclaration::Output(_) => "Output".into(),
        PortDeclaration::Ref(_) => "Ref".into(),
        PortDeclaration::Interface(_) => "Interface".into(),
    }
}

fn get_identifier_string<'a>(node: RefNode, syntax_tree: &'a SyntaxTree) -> Option<&'a str> {
    // unwrap_node! can take multiple types
    let locate = match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
        Some(RefNode::SimpleIdentifier(x)) => {
            Some(x.nodes.0)
        }
        Some(RefNode::EscapedIdentifier(x)) => {
            Some(x.nodes.0)
        }
        _ => None,
    }?;
    syntax_tree.get_str(&locate)
}