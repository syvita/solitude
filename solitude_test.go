package solitude

import (
	"testing"
)

var address = ":5050" //:7656

func TestCreateSession(test *testing.T) {
	_, err := NewSession("test_1", Datagram, address)

	if err != nil {
		test.Fatalf("failed to create session: %v", err)
	}
}

func TestAddress(test *testing.T) {
	session := Session{
		PublicKey: "wvodsusl7UzV2jE4HnTg0asK1FOM6LDSneK6B3YzA6gtsz7juIvNEfolcz4Z" +
			"tN011Vu3bDrEPbEfa43Jq392Phs7bvySwE9BHMcGJGu1mBD-~wAx67ofyZBWyx4BRbSuOCg" +
			"xWf9JMQLvpXuDlZBebdPiSO4MjHoks~julyoIIolE8anu3GkUeJk~FuMScVkyqaMjUiIEMx" +
			"~Jmx-WveR2HB3qdey3EjbwkX1guBD8zhreWQiWZyxg1-XuNQX3BWVyQBBuKKDRuY0S6ilpm" +
			"fRahe5AHoAm~w8OQ4Rbt749McE~sUyAPuiFdzXflaWCZORumUpuEh8qKfUH5jQCHFac-mML" +
			"F2L1pxzUPXQLD7EhbAuxtktIALf46dZn33Wj1iiIHCyiK-tqqTLQnVOLb-Yr5w-uKLihFHv" +
			"waKUrUEU~MrGRIkLGNJbKa~hdHJ3-TkEJHeU5rzb0iqRJ-GbgYPdBnulpkOq2kGNkm9~GEM" +
			"wbDhQhUCXgFbG8NY~RiAaZBIP5AAAA",
	}

	address, err := session.Address()

	if err != nil {
		test.Fatalf("failed to get address: %v", err)
	}

	if address != "V5WURQ4NSCBFBRVURQC7P7EQT6CXGE3QJ5PK6KSHVJKMZOJYBWQQ.b32.i2p" {
		test.Fatalf("address doesn't match, got: %s", address)
	}

	test.Logf("got address: %s", address)
}

func TestForwardDatagramSession(test *testing.T) {
	session, err := NewSession("test_2", Datagram, address)

	if err != nil {
		test.Fatalf("failed to create session: %v", err)
	}

	test.Logf("session: %+v", session)

	err = session.Forward(":2020")

	if err != nil {
		test.Fatalf("failed to forward session: %v", err)
	}
}

func TestForwardStreamSession(test *testing.T) {
	session, err := NewSession("test_3", Stream, address)

	if err != nil {
		test.Fatalf("failed to create session: %v", err)
	}

	test.Logf("session: %+v", session)

	err = session.Forward(":2020")

	if err != nil {
		test.Fatalf("failed to forward session: %v", err)
	}
}
