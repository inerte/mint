λdouble(x:ℤ)→ℤ=x*2
λisEven(x:ℤ)→𝔹=x%2=0
λsum(acc:ℤ,x:ℤ)→ℤ=acc+x

λmain()→ℤ=[1,2,3,4,5]↦double⊳isEven⊕sum⊕0
