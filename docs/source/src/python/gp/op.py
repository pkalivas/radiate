# --8<-- [start:ops]
import radiate as rd

add = rd.Op.add()
sub = rd.Op.sub()
mul = rd.Op.mul()
div = rd.Op.div()

constant = rd.Op.const(42.0)
variable = rd.Op.var(0)

sigmoid = rd.Op.sigmoid()
relu = rd.Op.relu()
tanh = rd.Op.tanh()

add_result = rd.Op.add().eval(1.0, 2.0)  # result is 3.0
const_result = rd.Op.const(42.0).eval()  # result is 42.0
var_result = rd.Op.var(0).eval(5.0, 10.0)  # result is 5.0 when evaluated with inputs
# --8<-- [end:ops]

# --8<-- [start:operation_mutator]
import radiate as rd

# Create a mutator that has a 10% chance to mutate an op and a 50% chance to replace it with a new one
mutator = rd.Mutate.op(0.1, 0.5)  # Using the dsl syntax for mutators
# --8<-- [end:operation_mutator]
