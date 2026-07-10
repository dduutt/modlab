package main

import (
	"context"
	"fmt"
	"io"
	"sync"
	"time"

	"github.com/dduutt/modbus"
	"github.com/wailsapp/wails/v2/pkg/runtime"
	gserial "go.bug.st/serial"
	"go.bug.st/serial/enumerator"
)

type ServerInstance struct {
	cancel context.CancelFunc
	store  *modbus.MemoryDataStore
	conn   io.Closer
	handle io.Closer
}

type ClientInstance struct {
	transport modbus.Transport
	conn      io.Closer
}

type App struct {
	ctx     context.Context
	clients sync.Map
	servers sync.Map
}

func NewApp() *App {
	return &App{}
}

// GetAvailablePorts scans the system for available serial ports
func (a *App) GetAvailablePorts() ([]string, error) {
	ports, err := enumerator.GetDetailedPortsList()
	if err != nil {
		return nil, err
	}
	var portNames []string
	for _, port := range ports {
		portNames = append(portNames, port.Name)
	}
	return portNames, nil
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
}

func (a *App) shutdown(ctx context.Context) {
	a.ClearAllConnections()
}

// ClearAllConnections forcefully stops all clients and servers. Useful for frontend hot-reloads.
func (a *App) ClearAllConnections() error {
	a.clients.Range(func(key, value interface{}) bool {
		ci := value.(*ClientInstance)
		ci.transport.Close()
		if ci.conn != nil {
			ci.conn.Close()
		}
		a.clients.Delete(key)
		return true
	})
	a.servers.Range(func(key, value interface{}) bool {
		si := value.(*ServerInstance)
		si.cancel()
		if si.handle != nil {
			si.handle.Close()
		}
		if si.conn != nil {
			si.conn.Close()
		}
		a.servers.Delete(key)
		return true
	})
	return nil
}

// ---------------------------------------------------------
// MASTER (CLIENT) LOGIC
// ---------------------------------------------------------

func getParity(p string) gserial.Parity {
	switch p {
	case "None", "N":
		return gserial.NoParity
	case "Even", "E":
		return gserial.EvenParity
	case "Odd", "O":
		return gserial.OddParity
	}
	return gserial.NoParity
}

func getStopBits(s uint) gserial.StopBits {
	switch s {
	case 1:
		return gserial.OneStopBit
	case 2:
		return gserial.TwoStopBits
	}
	return gserial.OneStopBit
}

// ConnectMaster creates and connects a Modbus Client
func (a *App) ConnectMaster(id string, protocol string, address string, rtuBaudRate uint, rtuDataBits uint, rtuParity string, rtuStopBits uint) error {
	a.DisconnectMaster(id)

	var transport modbus.Transport
	var conn io.ReadWriteCloser

	if protocol == "tcp" {
		transport = modbus.NewTCPTransport(address, modbus.WithTCPTimeout(2*time.Second))
	} else {
		mode := &gserial.Mode{
			BaudRate: int(rtuBaudRate),
			DataBits: int(rtuDataBits),
			Parity:   getParity(rtuParity),
			StopBits: getStopBits(rtuStopBits),
		}
		port, err := gserial.Open(address, mode)
		if err != nil {
			return err
		}
		conn = port
		transport = modbus.NewRTUTransport(port, modbus.WithRTUTimeout(2*time.Second))
	}

	a.clients.Store(id, &ClientInstance{
		transport: transport,
		conn:      conn,
	})
	return nil
}

// DisconnectMaster stops a Modbus Client
func (a *App) DisconnectMaster(id string) error {
	if val, ok := a.clients.Load(id); ok {
		ci := val.(*ClientInstance)
		ci.transport.Close()
		if ci.conn != nil {
			ci.conn.Close()
		}
		a.clients.Delete(id)
	}
	return nil
}

// ReadRegisters reads from the client
func (a *App) ReadRegisters(id string, unitId uint8, functionCode string, address uint16, count uint16) ([]uint16, error) {
	val, ok := a.clients.Load(id)
	if !ok {
		return nil, fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := modbus.NewClient(ci.transport, modbus.WithUnitID(unitId))
	ctx := context.Background()

	switch functionCode {
	case "03":
		return client.ReadHoldingRegisters(ctx, address, count)
	case "04":
		return client.ReadInputRegisters(ctx, address, count)
	case "01":
		bools, err := client.ReadCoils(ctx, address, count)
		if err != nil {
			return nil, err
		}
		res := make([]uint16, len(bools))
		for i, b := range bools {
			if b {
				res[i] = 1
			}
		}
		return res, nil
	case "02":
		bools, err := client.ReadDiscreteInputs(ctx, address, count)
		if err != nil {
			return nil, err
		}
		res := make([]uint16, len(bools))
		for i, b := range bools {
			if b {
				res[i] = 1
			}
		}
		return res, nil
	}
	return nil, fmt.Errorf("unsupported function code")
}

// WriteRegister writes to the client
func (a *App) WriteRegister(id string, unitId uint8, address uint16, value uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := modbus.NewClient(ci.transport, modbus.WithUnitID(unitId))
	ctx := context.Background()

	return client.WriteSingleRegister(ctx, address, value)
}

// WriteMultipleRegisters writes multiple registers to the client
func (a *App) WriteMultipleRegisters(id string, unitId uint8, address uint16, values []uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := modbus.NewClient(ci.transport, modbus.WithUnitID(unitId))
	ctx := context.Background()

	return client.WriteMultipleRegisters(ctx, address, values)
}

// WriteCoil writes a single coil to the client
func (a *App) WriteCoil(id string, unitId uint8, address uint16, value uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := modbus.NewClient(ci.transport, modbus.WithUnitID(unitId))
	ctx := context.Background()

	return client.WriteSingleCoil(ctx, address, value > 0)
}

// WriteMultipleCoils writes multiple coils to the client
func (a *App) WriteMultipleCoils(id string, unitId uint8, address uint16, values []uint16) error {
	val, ok := a.clients.Load(id)
	if !ok {
		return fmt.Errorf("client %s not connected", id)
	}

	ci := val.(*ClientInstance)
	client := modbus.NewClient(ci.transport, modbus.WithUnitID(unitId))
	ctx := context.Background()

	bools := make([]bool, len(values))
	for i, v := range values {
		bools[i] = (v > 0)
	}

	return client.WriteMultipleCoils(ctx, address, bools)
}

// ---------------------------------------------------------
// SLAVE (SERVER) LOGIC
// ---------------------------------------------------------

// WrappedDataStore intercepts writes to emit Wails events
type WrappedDataStore struct {
	store *modbus.MemoryDataStore
	ctx   context.Context
	id    string
}

func (w *WrappedDataStore) ReadCoils(address, quantity uint16) ([]bool, error) {
	return w.store.ReadCoils(address, quantity)
}

func (w *WrappedDataStore) ReadDiscreteInputs(address, quantity uint16) ([]bool, error) {
	return w.store.ReadDiscreteInputs(address, quantity)
}

func (w *WrappedDataStore) ReadHoldingRegisters(address, quantity uint16) ([]uint16, error) {
	return w.store.ReadHoldingRegisters(address, quantity)
}

func (w *WrappedDataStore) ReadInputRegisters(address, quantity uint16) ([]uint16, error) {
	return w.store.ReadInputRegisters(address, quantity)
}

func (w *WrappedDataStore) WriteCoils(address uint16, values []bool) error {
	err := w.store.WriteCoils(address, values)
	if err == nil {
		if w.ctx != nil {
			payload := make([]uint16, len(values))
			for i, v := range values {
				if v {
					payload[i] = 1
				}
			}
			runtime.EventsEmit(w.ctx, "slave_write", w.id, address, payload)
		}
	}
	return err
}

func (w *WrappedDataStore) WriteHoldingRegisters(address uint16, values []uint16) error {
	err := w.store.WriteHoldingRegisters(address, values)
	if err == nil {
		if w.ctx != nil {
			runtime.EventsEmit(w.ctx, "slave_write", w.id, address, values)
		}
	}
	return err
}

// StartSlave starts a Modbus Server
func (a *App) StartSlave(id string, protocol string, address string, rtuBaudRate uint, rtuDataBits uint, rtuParity string, rtuStopBits uint) error {
	a.StopSlave(id)

	store := modbus.NewMemoryDataStoreSized(65536, 65536, 65536, 65536)
	wstore := &WrappedDataStore{
		store: store,
		ctx:   a.ctx,
		id:    id,
	}

	handler := modbus.NewDataStoreHandler(wstore)

	ctx, cancel := context.WithCancel(context.Background())
	var conn io.ReadWriteCloser
	var handle io.Closer
	var err error

	if protocol == "tcp" {
		handle, err = modbus.StartTCPServer(ctx, address, handler)
		if err != nil {
			cancel()
			return err
		}
	} else {
		mode := &gserial.Mode{
			BaudRate: int(rtuBaudRate),
			DataBits: int(rtuDataBits),
			Parity:   getParity(rtuParity),
			StopBits: getStopBits(rtuStopBits),
		}
		port, err := gserial.Open(address, mode)
		if err != nil {
			cancel()
			return err
		}
		conn = port
		handle = modbus.StartRTUServer(ctx, port, handler)
	}

	a.servers.Store(id, &ServerInstance{
		cancel: cancel,
		store:  store,
		conn:   conn,
		handle: handle,
	})

	return nil
}

func (a *App) StopSlave(id string) error {
	if val, ok := a.servers.Load(id); ok {
		si := val.(*ServerInstance)
		si.cancel()
		if si.handle != nil {
			si.handle.Close()
		}
		if si.conn != nil {
			si.conn.Close()
		}
		a.servers.Delete(id)
	}
	return nil
}

// UpdateSlaveData allows the frontend to write data into the server memory manually
func (a *App) UpdateSlaveData(id string, address uint16, values []uint16, functionCode string) error {
	if val, ok := a.servers.Load(id); ok {
		si := val.(*ServerInstance)

		if functionCode == "01" || functionCode == "02" {
			bools := make([]bool, len(values))
			for i, v := range values {
				bools[i] = (v > 0)
			}
			if functionCode == "01" {
				return si.store.WriteCoils(address, bools)
			} else {
				return si.store.WriteDiscreteInputs(address, bools)
			}
		} else {
			if functionCode == "03" {
				return si.store.WriteHoldingRegisters(address, values)
			} else {
				return si.store.WriteInputRegisters(address, values)
			}
		}
	}
	return fmt.Errorf("slave memory not found")
}

// GetSlaveData allows the frontend to fetch current memory
func (a *App) GetSlaveData(id string, address uint16, count uint16, functionCode string) ([]uint16, error) {
	if val, ok := a.servers.Load(id); ok {
		si := val.(*ServerInstance)
		if functionCode == "01" || functionCode == "02" {
			var bools []bool
			var err error
			if functionCode == "01" {
				bools, err = si.store.ReadCoils(address, count)
			} else {
				bools, err = si.store.ReadDiscreteInputs(address, count)
			}
			if err != nil {
				return nil, err
			}
			res := make([]uint16, len(bools))
			for i, b := range bools {
				if b {
					res[i] = 1
				}
			}
			return res, nil
		} else {
			if functionCode == "03" {
				return si.store.ReadHoldingRegisters(address, count)
			} else {
				return si.store.ReadInputRegisters(address, count)
			}
		}
	}
	return nil, fmt.Errorf("slave memory not found")
}
