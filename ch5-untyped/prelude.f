
unit/;

/* logic */
tru = λx.λy. x;
fls = λx.λy. y;
not = λb. b fls tru;
and = λb.λc. b c fls;
or  = λb.λc. b tru c;
if-then-else = λb.λthen.λelse. b then else;

/* pair */
pair = λa.λb.λf. f a b;
fst  = λp. p tru;
snd  = λp. p fls;

/* natural number */
zro   = λs.λz. z;
iszro = λm. m (λx. fls) tru;
scc   = λn.λs.λz. s (n s z);
+     = λm.λn.λs.λz. m s (n s z);
*     = λm.λn.λs. m (n s);
**    = λm.λn. n m;

zz    = pair zro zro;
ss    = λp. pair (snd p) (scc (snd p));
prd   = λm. fst (m ss zz);

-     = λm.λn. n prd m;

nat-eq =λm.λn. and (iszro (m prd n)) (iszro (n prd m));

c0    = zro;
c1    = λs.λz. s z;
c2    = λs.λz. s (s z);
c3    = λs.λz. s (s (s z));
c4    = λs.λz. s (s (s (s z)));
c5    = λs.λz. s (s (s (s (s z))));

fix =λf. (λx. f (λy. x x y)) (λx. f (λy. x x y));

factorial = fix (λfct.λn. (iszro n) (λu.c1) (λu.(* n (fct (prd n)))) unit);
