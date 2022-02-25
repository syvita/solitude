package solitude

import (
	"bufio"
	"errors"
	"fmt"
	"net"
	"strings"
	"time"
)

type sessionStyle int

type Session struct {
	reader     *bufio.Reader
	connection net.Conn

	host string

	Style   sessionStyle
	Service string

	PublicKey  string
	PrivateKey string
}

// TODO: remove in go 1.18
type Any interface{}

const (
	Datagram sessionStyle = iota
	Stream
	Raw
)

func (style sessionStyle) String() string {
	styles := []string{
		"DATAGRAM",
		"STREAM",
		"RAW",
	}

	return styles[style]
}

func (session *Session) Hello() error {
	_, err := session.command("HELLO VERSION", map[string]Any{
		"min": 3.0,
		"max": 3.2,
	})

	if err != nil {
		return err
	}

	return nil
}

func (session *Session) Keys() error {
	result, err := session.command("DEST GENERATE", map[string]Any{})

	if err != nil {
		return err
	}

	session.PublicKey = result["PUB"]
	session.PrivateKey = result["PRIV"]

	return nil
}

func (session *Session) Forward(address string) error {
	split := strings.Split(address, ":")

	host := split[0]

	if host == "" {
		host = "127.0.0.1"
	}

	port := split[1]

	if session.Style == Datagram || session.Style == Raw {
		_, err := session.command("SESSION CREATE", map[string]Any{
			"style":       session.Style.String(),
			"ID":          session.Service,
			"destination": session.PrivateKey,
			"port":        port,
			"host":        host,
		})

		if err != nil {
			return err
		}
	} else {
		_, err := session.command("SESSION CREATE", map[string]Any{
			"style":       session.Style.String(),
			"ID":          session.Service,
			"destination": session.PrivateKey,
		})

		if err != nil {
			return err
		}

		go func() {
			forwardSession, err := NewSession(session.Service+"_forwarder", Stream, session.host)

			if err != nil {
				panic(err)
			}

			_, err = forwardSession.command("STREAM FORWARD", map[string]Any{
				"ID":   forwardSession.Service,
				"port": port,
				"host": host,
			})

			if err != nil {
				panic(err)
			}

			for {
				_, err = forwardSession.reader.ReadString('\n')

				if err != nil {
					panic(err)
				}

				time.Sleep(2 * time.Second)
			}
		}()
	}

	return nil
}

func (session *Session) command(statment string, variables map[string]Any) (map[string]string, error) {
	response := map[string]string{}

	var pairs []string

	for key, value := range variables {
		pairs = append(pairs, fmt.Sprintf("%s=%v", strings.ToUpper(key), value))
	}

	compiled := statment + " " + strings.Join(pairs, " ") + "\n"

	_, err := session.connection.Write([]byte(compiled))

	if err != nil {
		return response, err
	}

	plain, err := session.reader.ReadString('\n')

	if err != nil {
		return response, err
	}

	for _, cursor := range strings.Split(plain, " ") {
		split := strings.Split(cursor, "=")

		if len(split) != 2 {
			continue
		}

		response[split[0]] = split[1]
	}

	status, hasStatus := response["STATUS"]

	if hasStatus && status == "OK" {
		return response, errors.New("status is not OK")
	}

	return response, nil
}

func NewSession(service string, style sessionStyle, address string) (Session, error) {
	if style != Datagram && style != Stream && style != Raw {
		return Session{}, errors.New("session style must be `Datagram`, `Raw` or `Stream`")
	}

	connection, err := net.Dial("tcp", address)

	if err != nil {
		return Session{}, err
	}

	session := Session{
		reader:     bufio.NewReader(connection),
		connection: connection,

		host: address,

		Style:   style,
		Service: service,
	}

	err = session.Hello()

	if err != nil {
		return Session{}, fmt.Errorf("failed to greet: %w", err)
	}

	err = session.Keys()

	if err != nil {
		return Session{}, fmt.Errorf("failed to share keys: %w", err)
	}

	return session, nil
}
