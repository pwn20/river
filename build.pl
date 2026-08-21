#!/usr/bin/env perl

use strict;
use warnings;
use Time::HiRes qw(time);
use File::Copy;

# Flush standard output buffer immediately
$| = 1;

# ANSI color constants
my $YELLOW = "\e[1;33m";
my $RESET  = "\e[0m";
my $RED = "\e[4;31m";

# Directory to place compiled binaries (copied)
my $binary_target = 'binaries/';

# Fake enum to define build type
use constant
{
    BUILD_RELEASE => 0,
    BUILD_DEBUG => 1,
    BUILD_NO => 2, # Have to have this for the built-in menu seperator to prevent warnings
};

# Data structure associating IDs with their text and execution commands
my @menu_options = 
(
    {
        id => '1',
        text => 'Windows 10+ Release',
        cmd => 'cargo build -r',
        target => 'target/release/river.exe',
        binary_name => 'river_release.exe',
        build_type => BUILD_RELEASE,
    },
    {
        id => '2',
        text => 'Windows 10+ Debug',
        cmd => 'cargo build',
        target => 'target/debug/river.exe',
        binary_name => 'river_debug.exe',
        build_type => BUILD_DEBUG,
    },
    {
        id => '3',
        text => 'Linux Release',
        cmd => 'cargo linux --release',
        target => 'target/x86_64-unknown-linux-gnu/release/river',
        binary_name => 'river_linux_release',
        build_type => BUILD_RELEASE,
    },
    {
        id => '4',
        text => 'Linux Debug',
        cmd => 'cargo linux',
        target => 'target/x86_64-unknown-linux-gnu/debug/river',
        binary_name => 'river_linux_debug',
        build_type => BUILD_DEBUG,
    },
    {
        id => '5',
        text => 'Windows 7 Release',
        cmd => 'cargo win7r',
        target => 'target/x86_64-pc-windows-msvc/release/river.exe',
        binary_name => 'river_win7_release.exe',
        build_type => BUILD_RELEASE,
    },
    {
        id => '6',
        text => 'Windows 7 Debug',
        cmd => 'cargo win7d',
        target => 'target/x86_64-pc-windows-msvc/debug/river.exe',
        binary_name => 'river_win7_debug.exe',
        build_type => BUILD_DEBUG,
    },
    {
        id => '-',
        text => '-',
        build_type => BUILD_NO,
    },
    { 
        id    => '7', 
        text  => 'Build All Debug', 
        build_type => BUILD_DEBUG,
        bulk_build => 1,
    },
    { 
        id    => '8', 
        text  => 'Build All Release', 
        build_type => BUILD_RELEASE,
        bulk_build => 1,
    },
    {
        id => '-',
        text => '-',
        build_type => BUILD_NO,
    }
);

# Build a fast lookup hash for the input loop
my %menu_map;
foreach my $option (@menu_options)
{
    if ($option->{id} ne '-')
    {
        $menu_map{$option->{id}} = $option;
    }
}

sub execute_copy_binary
{
    my ($source, $target) = @_;
    copy($source, $binary_target.$target);
    print "${YELLOW}[+] copied $target to $binary_target${RESET}\n";
}

sub execute_build
{
    my ($text, $cmd, $target, $binary_name) = @_;
    
    print "\n${YELLOW}[+] Executing: $text ($cmd)${RESET}\n";
    print "----------------------------------------\n";
    
    my $start_time = time();
    my $exit_status = system($cmd);
    my $end_time = time();
    
    print "----------------------------------------\n";
    
    my $duration = sprintf("%.2f", $end_time - $start_time);
    my $real_code;
    
    if ($exit_status == -1)
    {
        print "[-] Execution failed: $!\n";
        $real_code = 1;
    }
    else
    {
        $real_code = $exit_status >> 8;
        
        if ($real_code == 0)
        {
            print "${YELLOW}[+] Build completed successfully in ${RESET}$duration ${YELLOW}seconds.${RESET}\n";
            execute_copy_binary($target, $binary_name);
        }
        else
        {
            print "${RED}[-] Build failed with exit code $real_code (Duration: $duration seconds).${RESET}\n";
        }
    }
    
    return $real_code;
}

while (1)
{
    print "\n";
    print "========================================\n";
    print " RIVER Build Menu\n";
    print "========================================\n";
    
    # Dynamically generate the menu from the data structure
    foreach my $option (@menu_options)
    {
        if ($option->{id} eq '-')
        {
            print " -\n";
        }
        else
        {
            print " $option->{id}.   $option->{text}\n";
        }
    }
    
    print " Q/E. Quit/Exit\n";
    print "========================================\n";
    print "Select an option: ";
    
    my $input = <STDIN>;
    
    # Handle EOF (e.g., Ctrl+D or piped input ending)
    if (!defined $input)
    {
        print "\nExiting...\n";
        exit(0);
    }
    
    chomp($input);
    
    # Trim leading and trailing whitespace
    $input =~ s/^\s+|\s+$//g;
    $input = lc($input);
    
    if (exists $menu_map{$input})
    {
        my $selection = $menu_map{$input};
        
        if (exists $selection->{cmd})
        {
            # Single command execution
            exit(execute_build($selection->{text}, $selection->{cmd}, $selection->{target}, $selection->{binary_name}));
        }
        elsif (exists $selection->{bulk_build})
        {
            # Chained 'Build All' execution

            # Iterate the menu looking for the build type
            foreach my $menu_item (@menu_options)
            {
                # Ignore the bulk build menus and the "-" spacer menu items
                if (!exists $menu_item->{bulk_build}) 
                {
                    # This individual build command is of the right build type (release/debug)
                    if ($menu_item->{build_type} == $selection->{build_type})
                    {
                        my $code = execute_build($menu_item->{text}, $menu_item->{cmd}, $menu_item->{target}, $menu_item->{binary_name});

                         if($code != 0)
                         {
                             print "\n[-] $menu_item->{text} aborted due to failure in: $menu_item->{text} ($menu_item->{cmd})\n";
                             exit($code);                            
                         }
                    }
                }
            }
         
            print "\n[+] $selection->{text} completed successfully!\n";
            exit(0);
        }
    }
    elsif ($input eq 'q' || $input eq 'e')
    {
        exit(0);
    }
    else
    {
        print "\n[!] Invalid input: '$input'. Please select a valid option from the menu.\n";
    }
}