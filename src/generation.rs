use crate::syntax::*;

const INITIAL_TEMPLATE: &str = r#"
.section .text
.globl _start
_start:
{}
pop %rax
call imprime_num
call sair
.include "runtime.s"
"#;

fn compute_stack(operator: &Operator, body: &mut String) {
    body.push_str(&format!("pop %rbx \n")); // right param
    body.push_str(&format!("pop %rax \n")); // left param
    match operator {
        Operator::Sum => {
            body.push_str(&format!("add %rbx, %rax \n"));
        }
        Operator::Sub => {
            body.push_str(&format!("sub %rbx, %rax \n"));
        }
        Operator::Div => {
            // idiv reg
            // multiplica o valor 128 bits formado pelos registradores RDX + RAX. o quociente inteiro fica em RAX e o resto em RDX. para não juntar RDX + RAX sempre use CQO para ignorar operando RDX.
            body.push_str(&format!("cqo \n"));
            body.push_str(&format!("idiv %rbx \n"));
        }
        Operator::Mul => {
            body.push_str(&format!("imul %rbx, %rax \n"));
        }
    }
    body.push_str(&format!("push %rax \n"));
}

fn evaluate_expression(expression: &Expression, body: &mut String) {
    match expression {
        Expression::NumberLiteral(number) => {
            body.push_str(&format!("push ${} \n", number));
        }
        Expression::BinOperation {
            left_value,
            operator,
            right_value,
        } => {
            evaluate_expression(left_value, body);
            evaluate_expression(right_value, body);
            compute_stack(operator, body);
        }
    }
}

pub fn generate_assembly(expression: Expression) -> String {
    let mut body: String = String::new();

    evaluate_expression(&expression, &mut body);

    return INITIAL_TEMPLATE.replace("{}", &body);
}
