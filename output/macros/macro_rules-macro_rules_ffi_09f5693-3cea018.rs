macro_rules ! CheckCondition { ($ cond : expr) => { unsafe { assert ! ($ cond , "{}: {}" , phase , stringify ! ($ cond)) ;}
} ; }