λreverse(s:𝕊,acc:𝕊)→𝕊≡s{
  ""→acc|
  s→reverse(s.tail(),s.head()+acc)
}

λis_palindrome(s:𝕊)→𝔹=s=reverse(s,"")

λmain()→𝕊="racecar is palindrome: "+is_palindrome("racecar")+" | hello is palindrome: "+is_palindrome("hello")
