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
