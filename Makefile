.PHONY: run clean

run:
	$(MAKE) -C front all
	$(MAKE) -C back run-web

clean:
	$(MAKE) -C front $@
	$(MAKE) -C back $@
