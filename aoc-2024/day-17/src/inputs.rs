use crate::Computer;

pub fn load_test() -> Computer
{
    return Computer { rA:          729,
                      rB:          0,
                      rC:          0,
                      instruction: 0,
                      program:     vec![0, 1, 5, 4, 3, 0],
                      clock:       0,
                      output:      vec![] };
}

pub fn load_input() -> Computer
{
    return Computer { rA:          59590048,
                      rB:          0,
                      rC:          0,
                      instruction: 0,
                      program:     vec![2,4,1,5,7,5,0,3,1,6,4,3,5,5,3,0],
                      clock:       0,
                      output:      vec![] };
}
