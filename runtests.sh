TESTS="stored fixed dynamic mixed"
cargo build
for i in $TESTS; do gzip -dc tests/$i.gz> tests/$i.old; done
for a in {0..1}; do 
  for b in {0..4}; do
    for k in {0..1}; do
      for s in {0..3}; do
        for i in $TESTS; do
          PARAM="-s$s -k$k -b$b -a$a"
          echo Testing $PARAM: $i
          RUST_BACKTRACE=1 ./target/debug/rgzip $PARAM \
            tests/$i.gz tests/$i.new > /dev/null
          diff -q tests/$i.old tests/$i.new
        done
      done
    done
  done
done
